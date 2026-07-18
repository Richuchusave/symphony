use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::time::Duration;
use tokio::sync::mpsc;

use crate::cache::CacheManager;
use crate::config::Config;
use crate::db::Database;
use crate::errors::*;
use crate::playback::{create_playback_engine, PlaybackEngine, PlaybackQueue};
use crate::plugin::PluginManager;
use crate::provider::{LocalProvider, MockProvider, ProviderRegistry, YouTubeProvider};
use crate::state::{create_shared_state, AppState, SharedState};
use crate::theme::Theme;
use crate::types::*;
use crate::ui::components::{PlayerBar, SearchInput, Sidebar};
use crate::ui::input::{handle_key, handle_mouse, Action};
use crate::ui::layout::AppLayout;
use crate::ui::screens;

pub struct Application {
    state: SharedState,
    engine: Box<dyn PlaybackEngine + Send>,
    queue: PlaybackQueue,
    registry: ProviderRegistry,
    plugin_manager: PluginManager,
    db: Option<Database>,
    #[allow(dead_code)]
    cache: Option<CacheManager>,
    #[allow(dead_code)]
    theme: Theme,
    layout: AppLayout,
    sidebar: Sidebar,
    player_bar: PlayerBar,
    #[allow(dead_code)]
    search_input: SearchInput,
    event_rx: mpsc::Receiver<AppEvent>,
    event_tx: mpsc::Sender<AppEvent>,
    tick_interval: Duration,
}

#[allow(clippy::await_holding_lock)]
impl Application {
    pub async fn new(config: Config) -> Result<Self> {
        config.ensure_dirs()?;

        let state = create_shared_state(config.clone());
        {
            let mut s = state.write();
            s.config = config.clone();
        }

        let theme = Theme::from_name(&config.theme.name).unwrap_or_else(|_| Theme::default());

        let mut engine = create_playback_engine(&config);
        engine.set_volume(config.playback.volume);

        let mut queue = PlaybackQueue::new();

        let mut registry = ProviderRegistry::new();
        registry.register(Box::new(YouTubeProvider::new(
            config.providers.youtube_music.clone().unwrap_or_default(),
        )));
        registry.register(Box::new(MockProvider::new()));
        {
            let local_config = config.providers.local.clone();
            let mut local = LocalProvider::new(&local_config);
            let _ = local.scan();
            registry.register(Box::new(local));
        }

        let requested_provider = config.general.default_provider.clone();
        if registry.set_active(&requested_provider).is_err() {
            let fallback = "mock".to_string();
            let _ = registry.set_active(&fallback);
        }

        let initial_library = registry.active().search("", 10_000, 0).await.ok();

        {
            let mut s = state.write();
            s.available_providers = registry.list_ids();
            s.active_provider = registry.active().id();
            s.playback.volume = config.playback.volume.clamp(0.0, 1.0);
            if let Some(results) = initial_library {
                s.library.tracks = results
                    .tracks
                    .into_iter()
                    .map(|track| (track.id.clone(), track))
                    .collect();
                s.library.albums = results
                    .albums
                    .into_iter()
                    .map(|album| (album.id.clone(), album))
                    .collect();
                s.library.artists = results
                    .artists
                    .into_iter()
                    .map(|artist| (artist.id.clone(), artist))
                    .collect();
                s.library.playlists = results
                    .playlists
                    .into_iter()
                    .map(|playlist| (playlist.id.clone(), playlist))
                    .collect();
            }
        }

        let mut plugin_manager = PluginManager::new();
        let _ = plugin_manager.register(Box::new(crate::plugin::manager::SimpleLoggerPlugin));
        let _ = plugin_manager.register(Box::new(crate::plugin::manager::TickCounterPlugin::new()));

        let db = Database::new(&config.general.data_dir.join("symphony.db")).ok();

        let cache = Some(CacheManager::new(
            config.general.cache_dir.clone(),
            config.cache.max_size_mb,
            config.cache.ttl_hours,
        ));

        if let Some(ref db) = db {
            if let Ok((track_ids, saved_index)) = db.load_queue() {
                if !track_ids.is_empty() {
                    for tid in &track_ids {
                        queue.add(tid.clone());
                    }
                    if let Some(idx) = saved_index {
                        let _ = queue.play_index(idx);
                    }
                    {
                        let mut s = state.write();
                        s.queue = track_ids;
                        s.queue_index = saved_index;
                    }
                }
            }
        }

        let (event_tx, event_rx) = mpsc::channel(256);

        let layout = AppLayout {
            sidebar_visible: true,
        };

        let mut plugin_mgr = plugin_manager;
        let state_guard = state.read();
        plugin_mgr.initialize_all(&state_guard).await;
        drop(state_guard);

        let app = Self {
            state,
            engine,
            queue,
            registry,
            plugin_manager: plugin_mgr,
            db,
            cache,
            theme,
            layout,
            sidebar: Sidebar::new(),
            player_bar: PlayerBar,
            search_input: SearchInput,
            event_rx,
            event_tx,
            tick_interval: Duration::from_millis(50),
        };

        Ok(app)
    }

    pub async fn run(&mut self) -> Result<()> {
        let (mut terminal, terminal_guard) = init_terminal()?;

        let run_result: Result<()> = async {
            let mut last_tick = tokio::time::Instant::now();

            loop {
                let state = self.state.read().clone();
                terminal.draw(|f| {
                    self.render(f, &state);
                })?;

                let tick_duration = self.tick_interval;

                if event::poll(
                    tick_duration
                        .checked_sub(tokio::time::Instant::now().duration_since(last_tick))
                        .unwrap_or(Duration::ZERO),
                )? {
                    match event::read()? {
                        Event::Key(key) => {
                            let search_focused = self.state.read().search_focused;
                            let action = if search_focused {
                                match (key.modifiers, key.code) {
                                    (
                                        KeyModifiers::NONE | KeyModifiers::SHIFT,
                                        KeyCode::Char(c),
                                    ) => Action::Input(c),
                                    _ => handle_key(key),
                                }
                            } else {
                                handle_key(key)
                            };
                            if self.handle_action(action).await {
                                break;
                            }
                        }
                        Event::Mouse(mouse) => {
                            let action = handle_mouse(mouse);
                            if self.handle_action(action).await {
                                break;
                            }
                        }
                        Event::Resize(w, h) => {
                            self.dispatch_event(AppEvent::Resize(w, h)).await;
                        }
                        _ => {}
                    }
                }

                if last_tick.elapsed() >= tick_duration {
                    self.tick().await;
                    last_tick = tokio::time::Instant::now();
                }

                self.process_events().await;
            }
            Ok(())
        }
        .await;

        drop(terminal_guard);
        self.shutdown().await;
        run_result
    }

    fn render(&self, f: &mut ratatui::Frame, state: &AppState) {
        let rects = self.layout.render_area(f.area(), &state.config);

        if self.layout.sidebar_visible {
            self.sidebar.render(f, rects.sidebar, state);
        }

        screens::render_screen(f, rects.main, state);

        self.player_bar.render(f, rects.player_bar, state);
    }

    async fn handle_action(&mut self, action: Action) -> bool {
        match action {
            Action::Quit => return true,
            Action::PlayPause => self.cmd_play_pause().await,
            Action::Stop => self.cmd_stop().await,
            Action::Next => self.cmd_next().await,
            Action::Previous => self.cmd_previous().await,
            Action::VolumeUp => self.cmd_volume_up().await,
            Action::VolumeDown => self.cmd_volume_down().await,
            Action::SeekForward(secs) => self.cmd_seek(secs).await,
            Action::SeekBackward(secs) => self.cmd_seek(-secs).await,
            Action::Search | Action::FocusSearch => self.cmd_focus_search(),
            Action::Enter => self.cmd_enter().await,
            Action::Back => self.cmd_back(),
            Action::ToggleSidebar => self.cmd_toggle_sidebar(),
            Action::SelectNext => self.cmd_select_next(),
            Action::SelectPrevious => self.cmd_select_previous(),
            Action::ScrollUp(..) => self.cmd_select_previous(),
            Action::ScrollDown(..) => self.cmd_select_next(),
            Action::ToggleShuffle => self.cmd_toggle_shuffle(),
            Action::ToggleRepeat => self.cmd_toggle_repeat(),
            Action::ShowQueue => self.cmd_show_queue(),
            Action::Help => self.cmd_help(),
            Action::Input(c) => {
                let mut state = self.state.write();
                if state.search_focused {
                    state.search_query.push(c);
                }
            }
            Action::Delete => {
                let mut state = self.state.write();
                if state.search_focused {
                    state.search_query.pop();
                }
            }
            Action::MouseClick(x, y) => {
                let (width, height) = crossterm::terminal::size().unwrap_or((80, 24));
                let rects = self.layout.render_area(
                    ratatui::prelude::Rect::new(0, 0, width, height),
                    &self.state.read().config,
                );
                if self.layout.sidebar_visible {
                    if let Some(screen) = self.sidebar.handle_click(x, y, rects.sidebar) {
                        let mut s = self.state.write();
                        s.navigate_to(screen);
                        return false;
                    }
                }
                let pb_action = {
                    let state = self.state.read();
                    self.player_bar.handle_click(x, y, rects.player_bar, &state)
                };
                if let Some(a) = pb_action {
                    match a {
                        Action::PlayPause => self.cmd_play_pause().await,
                        Action::VolumeUp => self.cmd_volume_up().await,
                        Action::VolumeDown => self.cmd_volume_down().await,
                        _ => {}
                    }
                }
            }
            Action::None => {}
            _ => {}
        }
        false
    }

    async fn process_events(&mut self) {
        while let Ok(event) = self.event_rx.try_recv() {
            self.plugin_manager
                .dispatch_event(&event, &mut self.state.write())
                .await;

            match event {
                AppEvent::TrackChanged(track_id) => {
                    let mut state = self.state.write();
                    state.playback.current_track_id = Some(track_id);
                }
                AppEvent::PlaybackUpdate(pb) => {
                    let mut state = self.state.write();
                    state.playback = pb;
                }
                AppEvent::SearchComplete(results) => {
                    let mut state = self.state.write();
                    state.search_results = results;
                    state.is_searching = false;
                }
                AppEvent::Notification(msg) => {
                    let mut state = self.state.write();
                    state.notify(msg);
                }
                AppEvent::Error(msg) => {
                    let mut state = self.state.write();
                    state.notify(format!("Error: {msg}"));
                }
                _ => {}
            }
        }
    }

    async fn tick(&mut self) {
        self.plugin_manager.tick_all(&mut self.state.write()).await;

        {
            let mut state = self.state.write();
            state.playback.position = self.engine.position();
            state.playback.duration = self.engine.duration();
            state.playback.status = self.engine.status();
        }
    }

    async fn dispatch_event(&self, event: AppEvent) {
        let _ = self.event_tx.send(event).await;
    }

    // ── Commands ─────────────────────────────────────────────────────────

    async fn cmd_play_pause(&mut self) {
        let track_id = {
            let state = self.state.read();
            state.playback.current_track_id.clone()
        };

        match self.engine.status() {
            PlaybackStatus::Playing => {
                let _ = self.engine.pause().await;
            }
            PlaybackStatus::Paused => {
                let _ = self.engine.resume().await;
            }
            PlaybackStatus::Stopped => {
                if let Some(tid) = track_id {
                    self.play_track(tid).await;
                } else if !self.queue.is_empty() {
                    if let Some(tid) = self.queue.current().cloned() {
                        self.play_track(tid).await;
                    }
                }
            }
        }
    }

    async fn cmd_stop(&mut self) {
        let _ = self.engine.stop().await;
        let mut state = self.state.write();
        state.playback.status = PlaybackStatus::Stopped;
        state.playback.position = Duration::ZERO;
        state.playback.current_track_id = None;
    }

    async fn cmd_next(&mut self) {
        if let Some(track_id) = self.queue.next() {
            {
                let mut state = self.state.write();
                state.queue = self.queue.tracks.clone();
                state.queue_index = self.queue.current_index;
            }
            self.play_track(track_id).await;
        }
    }

    async fn cmd_previous(&mut self) {
        if let Some(track_id) = self.queue.previous() {
            {
                let mut state = self.state.write();
                state.queue = self.queue.tracks.clone();
                state.queue_index = self.queue.current_index;
            }
            self.play_track(track_id).await;
        }
    }

    async fn cmd_volume_up(&mut self) {
        let mut state = self.state.write();
        let step = state.config.playback.volume_step;
        let new_vol = (state.playback.volume + step).min(1.0);
        state.playback.volume = new_vol;
        self.engine.set_volume(new_vol);
    }

    async fn cmd_volume_down(&mut self) {
        let mut state = self.state.write();
        let step = state.config.playback.volume_step;
        let new_vol = (state.playback.volume - step).max(0.0);
        state.playback.volume = new_vol;
        self.engine.set_volume(new_vol);
    }

    async fn cmd_seek(&mut self, secs: f64) {
        let current = self.engine.position();
        let duration = self.engine.duration();
        let new_pos = if secs >= 0.0 {
            (current + Duration::from_secs_f64(secs)).min(duration)
        } else {
            current.saturating_sub(Duration::from_secs_f64(-secs))
        };
        let _ = self.engine.seek(new_pos).await;
    }

    fn cmd_focus_search(&mut self) {
        let mut state = self.state.write();
        state.search_focused = !state.search_focused;
        if state.search_focused {
            state.navigate_to(Screen::Search);
        }
    }

    async fn cmd_enter(&mut self) {
        let (screen, search_focused, query, provider_id, selected_index) = {
            let state = self.state.read();
            (
                state.current_screen.clone(),
                state.search_focused,
                state.search_query.clone(),
                state.active_provider.clone(),
                state.selected_index,
            )
        };

        match screen {
            Screen::Search if search_focused => {
                self.state.write().search_focused = false;
                if !query.trim().is_empty() {
                    self.search(query, provider_id).await;
                }
            }
            Screen::Search => {
                let track = self
                    .state
                    .read()
                    .search_results
                    .tracks
                    .get(selected_index)
                    .cloned();
                if let Some(track) = track {
                    self.enqueue_and_play(track).await;
                }
            }
            Screen::Library => {
                let mut tracks: Vec<_> =
                    self.state.read().library.tracks.values().cloned().collect();
                tracks.sort_by(|a, b| a.artist.cmp(&b.artist).then(a.title.cmp(&b.title)));
                if let Some(track) = tracks.get(selected_index).cloned() {
                    self.enqueue_and_play(track).await;
                }
            }
            Screen::Queue => {
                if let Some(track_id) = self.queue.play_index(selected_index).cloned() {
                    {
                        let mut state = self.state.write();
                        state.queue = self.queue.tracks.clone();
                        state.queue_index = self.queue.current_index;
                    }
                    self.play_track(track_id).await;
                }
            }
            _ => {}
        }
    }

    async fn enqueue_and_play(&mut self, track: Track) {
        let index = self
            .queue
            .tracks
            .iter()
            .position(|track_id| track_id == &track.id)
            .unwrap_or_else(|| {
                self.queue.add(track.id.clone());
                self.queue.len() - 1
            });
        let _ = self.queue.play_index(index);

        {
            let mut state = self.state.write();
            state.track_cache.insert(track.id.clone(), track.clone());
            state.queue = self.queue.tracks.clone();
            state.queue_index = self.queue.current_index;
        }

        self.play_track(track.id).await;
    }

    fn cmd_back(&mut self) {
        let mut state = self.state.write();
        if state.search_focused {
            state.search_focused = false;
        } else {
            state.navigate_back();
        }
    }

    fn cmd_toggle_sidebar(&mut self) {
        self.layout.sidebar_visible = !self.layout.sidebar_visible;
    }

    fn cmd_select_next(&mut self) {
        let mut state = self.state.write();
        state.sidebar_focused = false;
        let max = match state.current_screen {
            Screen::Search => state.search_results.tracks.len(),
            Screen::Library => state.library.track_count(),
            Screen::Albums | Screen::AlbumDetail(_) => state.library.album_count(),
            Screen::Artists | Screen::ArtistDetail(_) => state.library.artist_count(),
            Screen::Playlists | Screen::PlaylistDetail(_) => state.library.playlist_count(),
            Screen::Queue => state.queue.len(),
            _ => 0,
        };
        let max = max.max(1);
        state.select_next(max);
    }

    fn cmd_select_previous(&mut self) {
        let mut state = self.state.write();
        state.sidebar_focused = false;
        let max = match state.current_screen {
            Screen::Search => state.search_results.tracks.len(),
            Screen::Library => state.library.track_count(),
            Screen::Albums | Screen::AlbumDetail(_) => state.library.album_count(),
            Screen::Artists | Screen::ArtistDetail(_) => state.library.artist_count(),
            Screen::Playlists | Screen::PlaylistDetail(_) => state.library.playlist_count(),
            Screen::Queue => state.queue.len(),
            _ => 0,
        };
        let max = max.max(1);
        state.select_previous(max);
    }

    fn cmd_toggle_shuffle(&mut self) {
        self.queue.toggle_shuffle();
        let shuffle = self.queue.shuffle;
        let mut state = self.state.write();
        state.playback.shuffle = shuffle;
        let msg = format!("Shuffle: {}", state.playback.shuffle);
        state.notify(msg);
    }

    fn cmd_toggle_repeat(&mut self) {
        self.queue.toggle_repeat();
        let repeat = self.queue.repeat.clone();
        let mut state = self.state.write();
        state.playback.repeat = repeat;
        let msg = format!("Repeat: {}", state.playback.repeat);
        state.notify(msg);
    }

    fn cmd_show_queue(&mut self) {
        let mut state = self.state.write();
        state.navigate_to(Screen::Queue);
    }

    fn cmd_help(&mut self) {
        let mut state = self.state.write();
        state.navigate_to(Screen::Help);
    }

    async fn search(&self, query: String, provider_id: ProviderId) {
        let mut state = self.state.write();
        state.is_searching = true;
        state.search_query = query.clone();
        drop(state);

        if let Some(provider) = self.registry.get(&provider_id) {
            match provider.search(&query, 50, 0).await {
                Ok(results) => {
                    self.dispatch_event(AppEvent::SearchComplete(results)).await;
                }
                Err(e) => {
                    self.dispatch_event(AppEvent::Error(e.to_string())).await;
                }
            }
        } else {
            self.dispatch_event(AppEvent::Error(format!(
                "Provider '{provider_id}' is not available"
            )))
            .await;
        }
    }

    async fn play_track(&mut self, track_id: TrackId) {
        let provider_id = self.state.read().active_provider.clone();
        let Some(provider) = self.registry.get(&provider_id) else {
            self.dispatch_event(AppEvent::Error(format!(
                "Provider '{provider_id}' is not available"
            )))
            .await;
            return;
        };
        let track = match provider.track(&track_id).await {
            Ok(track) => track,
            Err(error) => {
                self.dispatch_event(AppEvent::Error(error.to_string()))
                    .await;
                return;
            }
        };
        let stream_url = match provider.resolve_stream_url(&track).await {
            Ok(url) => url,
            Err(error) => {
                self.dispatch_event(AppEvent::Error(error.to_string()))
                    .await;
                return;
            }
        };

        match self.engine.play(&track_id, &stream_url).await {
            Ok(()) => {
                let mut state = self.state.write();
                state.playback.status = PlaybackStatus::Playing;
                state.playback.current_track_id = Some(track_id.clone());
                state.playback.position = Duration::ZERO;
                state.queue = self.queue.tracks.clone();
                state.queue_index = self.queue.current_index;
                state.track_cache.insert(track_id.clone(), track);

                drop(state);
                self.dispatch_event(AppEvent::TrackChanged(track_id)).await;
            }
            Err(error) => {
                self.dispatch_event(AppEvent::Error(error.to_string()))
                    .await;
            }
        }
    }

    async fn shutdown(&mut self) {
        self.plugin_manager.shutdown_all().await;

        if let Some(ref db) = self.db {
            let _ = db.save_queue(&self.queue.tracks, self.queue.current_index);
            let state = self.state.read();
            let _ = db.save_setting("volume", &state.playback.volume.to_string());
        }

        let _ = self.engine.stop().await;
    }
}

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = restore_terminal();
    }
}

fn init_terminal() -> Result<(Terminal<CrosstermBackend<io::Stdout>>, TerminalGuard)> {
    crossterm::terminal::enable_raw_mode()?;
    let guard = TerminalGuard;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    crossterm::execute!(stdout, crossterm::event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    terminal.hide_cursor()?;
    Ok((terminal, guard))
}

fn restore_terminal() -> Result<()> {
    crossterm::execute!(
        io::stdout(),
        crossterm::event::DisableMouseCapture,
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::cursor::Show,
    )?;
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> (Config, std::path::PathBuf) {
        let root = std::env::temp_dir().join(format!(
            "symphony-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut config = Config::default();
        config.general.data_dir = root.join("data");
        config.general.config_dir = root.join("config");
        config.general.cache_dir = root.join("cache");
        config.general.default_provider = "mock".to_string();
        config.providers.local.music_directories.clear();
        (config, root)
    }

    #[tokio::test]
    async fn search_result_can_be_selected_and_played() {
        let (config, root) = test_config();
        let mut app = Application::new(config).await.unwrap();

        assert_eq!(app.state.read().active_provider, "mock");
        assert!(!app.state.read().library.tracks.is_empty());

        app.cmd_focus_search();
        for character in "Digital".chars() {
            app.handle_action(Action::Input(character)).await;
        }
        app.cmd_enter().await;
        app.process_events().await;

        assert!(!app.state.read().search_results.tracks.is_empty());
        app.cmd_enter().await;

        {
            let state = app.state.read();
            assert_eq!(state.playback.status, PlaybackStatus::Playing);
            assert_eq!(state.queue.len(), 1);
            assert_eq!(state.queue_index, Some(0));
        }

        app.shutdown().await;
        drop(app);
        let _ = std::fs::remove_dir_all(root);
    }
}
