use crate::config::Config;
use crate::types::*;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

pub type SharedState = Arc<RwLock<AppState>>;

#[derive(Debug, Clone)]
pub struct AppState {
    // Navigation
    pub current_screen: Screen,
    pub screen_history: Vec<Screen>,

    // Search
    pub search_query: String,
    pub search_results: SearchResults,
    pub is_searching: bool,
    pub search_focused: bool,

    // Library
    pub library: Library,

    // Queue
    pub queue: Vec<TrackId>,
    pub queue_index: Option<usize>,
    pub queue_history: Vec<TrackId>,

    // Playback
    pub playback: PlaybackState,

    // Providers
    pub active_provider: ProviderId,
    pub available_providers: Vec<ProviderId>,

    // UI state
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub sidebar_focused: bool,
    pub notification: Option<(String, chrono::DateTime<chrono::Utc>)>,

    // Config
    pub config: Config,

    // Track cache (resolved track details)
    pub track_cache: HashMap<TrackId, Track>,
    pub album_cache: HashMap<AlbumId, Album>,
    pub artist_cache: HashMap<ArtistId, Artist>,
    pub playlist_cache: HashMap<PlaylistId, Playlist>,

    // Volume
    pub volume_before_mute: f64,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_screen: Screen::Home,
            screen_history: Vec::new(),
            search_query: String::new(),
            search_results: SearchResults {
                tracks: Vec::new(),
                albums: Vec::new(),
                artists: Vec::new(),
                playlists: Vec::new(),
                query: String::new(),
                total_results: 0,
            },
            is_searching: false,
            search_focused: false,
            library: Library::default(),
            queue: Vec::new(),
            queue_index: None,
            queue_history: Vec::new(),
            playback: PlaybackState::default(),
            active_provider: "mock".to_string(),
            available_providers: vec!["mock".to_string(), "local".to_string()],
            selected_index: 0,
            scroll_offset: 0,
            sidebar_focused: true,
            notification: None,
            config: Config::default(),
            track_cache: HashMap::new(),
            album_cache: HashMap::new(),
            artist_cache: HashMap::new(),
            playlist_cache: HashMap::new(),
            volume_before_mute: 0.8,
        }
    }
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    // ─── Navigation ──────────────────────────────────────────────────────

    pub fn navigate_to(&mut self, screen: Screen) {
        if self.current_screen != screen {
            let current = std::mem::replace(&mut self.current_screen, screen);
            self.screen_history.push(current);
            self.selected_index = 0;
            self.scroll_offset = 0;
        }
    }

    pub fn navigate_back(&mut self) {
        if let Some(previous) = self.screen_history.pop() {
            self.current_screen = previous;
            self.selected_index = 0;
            self.scroll_offset = 0;
        }
    }

    pub fn can_navigate_back(&self) -> bool {
        !self.screen_history.is_empty()
    }

    // ─── Queue ───────────────────────────────────────────────────────────

    pub fn current_track_in_queue(&self) -> Option<(usize, &TrackId)> {
        self.queue_index
            .and_then(|i| self.queue.get(i))
            .map(|id| (self.queue_index.unwrap(), id))
    }

    pub fn next_track(&mut self) -> Option<TrackId> {
        match self.playback.repeat {
            RepeatMode::One => {
                return self.queue_index.and_then(|i| self.queue.get(i).cloned());
            }
            RepeatMode::All => {
                if let Some(i) = self.queue_index {
                    let next = (i + 1) % self.queue.len();
                    self.queue_index = Some(next);
                    return self.queue.get(next).cloned();
                }
            }
            RepeatMode::Off => {
                if let Some(i) = self.queue_index {
                    let next = i + 1;
                    if next < self.queue.len() {
                        self.queue_index = Some(next);
                        return self.queue.get(next).cloned();
                    }
                }
            }
        }
        None
    }

    pub fn previous_track(&mut self) -> Option<TrackId> {
        if let Some(i) = self.queue_index {
            if i > 0 {
                let prev = i - 1;
                self.queue_index = Some(prev);
                return self.queue.get(prev).cloned();
            }
        }
        None
    }

    pub fn add_to_queue(&mut self, track_id: TrackId) {
        self.queue.push(track_id);
        if self.queue_index.is_none() && self.queue.len() == 1 {
            self.queue_index = Some(0);
        }
    }

    pub fn remove_from_queue(&mut self, index: usize) {
        if index < self.queue.len() {
            self.queue.remove(index);
            if let Some(current) = self.queue_index {
                if index < current {
                    self.queue_index = Some(current - 1);
                } else if index == current {
                    self.queue_index = if current >= self.queue.len() {
                        if self.queue.is_empty() {
                            None
                        } else {
                            Some(self.queue.len() - 1)
                        }
                    } else {
                        Some(current)
                    };
                }
            }
        }
    }

    pub fn clear_queue(&mut self) {
        self.queue.clear();
        self.queue_index = None;
        self.queue_history.clear();
    }

    pub fn move_in_queue(&mut self, from: usize, to: usize) {
        if from < self.queue.len() && to < self.queue.len() {
            let track = self.queue.remove(from);
            self.queue.insert(to, track);
            if let Some(current) = self.queue_index {
                if from == current {
                    self.queue_index = Some(to);
                } else if from < current && to >= current {
                    self.queue_index = Some(current - 1);
                } else if from > current && to <= current {
                    self.queue_index = Some(current + 1);
                }
            }
        }
    }

    // ─── Playback ────────────────────────────────────────────────────────

    pub fn current_track(&self) -> Option<&Track> {
        self.playback.current_track_id.as_ref().and_then(|id| {
            self.track_cache
                .get(id)
                .or_else(|| self.library.tracks.get(id))
        })
    }

    // ─── Notifications ───────────────────────────────────────────────────

    pub fn notify(&mut self, message: impl Into<String>) {
        self.notification = Some((message.into(), chrono::Utc::now()));
    }

    // ─── Selection ───────────────────────────────────────────────────────

    pub fn select_next(&mut self, max: usize) {
        if max > 0 {
            self.selected_index = (self.selected_index + 1).min(max - 1);
            self.adjust_scroll(max);
        }
    }

    pub fn select_previous(&mut self, max: usize) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.adjust_scroll(max);
        }
    }

    pub fn adjust_scroll(&mut self, _max: usize) {
        let visible = 20;
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + visible {
            self.scroll_offset = self.selected_index - visible + 1;
        }
    }
}

pub fn create_shared_state(config: Config) -> SharedState {
    Arc::new(RwLock::new(AppState::new(config)))
}
