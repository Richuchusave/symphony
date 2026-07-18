use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::time::Duration;

// Type aliases
pub type TrackId = String;
pub type AlbumId = String;
pub type ArtistId = String;
pub type PlaylistId = String;
pub type ProviderId = String;
pub type UserId = String;

// ─── Core Models ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Track {
    pub id: TrackId,
    pub title: String,
    pub artist: String,
    pub artist_id: ArtistId,
    pub album: String,
    pub album_id: AlbumId,
    pub duration: Duration,
    pub track_number: u32,
    pub disc_number: u32,
    pub genre: String,
    pub year: i32,
    pub cover_url: Option<String>,
    pub stream_url: Option<String>,
    pub file_path: Option<String>,
    pub provider: ProviderId,
    pub is_local: bool,
}

impl Track {
    pub fn duration_formatted(&self) -> String {
        let secs = self.duration.as_secs();
        let mins = secs / 60;
        let secs = secs % 60;
        format!("{}:{:02}", mins, secs)
    }

    pub fn display_title(&self) -> String {
        format!("{} — {}", self.artist, self.title)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Album {
    pub id: AlbumId,
    pub title: String,
    pub artist: String,
    pub artist_id: ArtistId,
    pub tracks: Vec<TrackId>,
    pub cover_url: Option<String>,
    pub year: i32,
    pub track_count: u32,
    pub duration: Duration,
    pub provider: ProviderId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Artist {
    pub id: ArtistId,
    pub name: String,
    pub image_url: Option<String>,
    pub genres: Vec<String>,
    pub album_count: u32,
    pub provider: ProviderId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Playlist {
    pub id: PlaylistId,
    pub name: String,
    pub description: String,
    pub tracks: Vec<TrackId>,
    pub cover_url: Option<String>,
    pub owner: UserId,
    pub provider: ProviderId,
    pub is_user_created: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── Search ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchResults {
    pub tracks: Vec<Track>,
    pub albums: Vec<Album>,
    pub artists: Vec<Artist>,
    pub playlists: Vec<Playlist>,
    pub query: String,
    pub total_results: u32,
}

// ─── Queue ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepeatMode {
    Off,
    One,
    All,
}

impl fmt::Display for RepeatMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RepeatMode::Off => write!(f, "Off"),
            RepeatMode::One => write!(f, "One"),
            RepeatMode::All => write!(f, "All"),
        }
    }
}

impl RepeatMode {
    pub fn cycle(&self) -> Self {
        match self {
            RepeatMode::Off => RepeatMode::All,
            RepeatMode::All => RepeatMode::One,
            RepeatMode::One => RepeatMode::Off,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShuffleMode {
    Off,
    On,
}

impl fmt::Display for ShuffleMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShuffleMode::Off => write!(f, "Off"),
            ShuffleMode::On => write!(f, "On"),
        }
    }
}

// ─── Playback ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackStatus {
    Stopped,
    Playing,
    Paused,
}

impl fmt::Display for PlaybackStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlaybackStatus::Stopped => write!(f, "⏹"),
            PlaybackStatus::Playing => write!(f, "▶"),
            PlaybackStatus::Paused => write!(f, "⏸"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlaybackState {
    pub status: PlaybackStatus,
    pub current_track_id: Option<TrackId>,
    pub position: Duration,
    pub duration: Duration,
    pub volume: f64,
    pub repeat: RepeatMode,
    pub shuffle: ShuffleMode,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            status: PlaybackStatus::Stopped,
            current_track_id: None,
            position: Duration::ZERO,
            duration: Duration::ZERO,
            volume: 0.8,
            repeat: RepeatMode::Off,
            shuffle: ShuffleMode::Off,
        }
    }
}

impl PlaybackState {
    pub fn position_formatted(&self) -> String {
        format_duration(self.position)
    }

    pub fn duration_formatted(&self) -> String {
        format_duration(self.duration)
    }

    pub fn progress_pct(&self) -> f64 {
        if self.duration.as_secs_f64() > 0.0 {
            (self.position.as_secs_f64() / self.duration.as_secs_f64()).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

// ─── Navigation ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Screen {
    Home,
    Search,
    Library,
    Albums,
    AlbumDetail(AlbumId),
    Artists,
    ArtistDetail(ArtistId),
    Playlists,
    PlaylistDetail(PlaylistId),
    Queue,
    Downloads,
    Settings,
    Help,
}

impl Screen {
    pub fn title(&self) -> &str {
        match self {
            Screen::Home => "Home",
            Screen::Search => "Search",
            Screen::Library => "Library",
            Screen::Albums => "Albums",
            Screen::AlbumDetail(_) => "Album",
            Screen::Artists => "Artists",
            Screen::ArtistDetail(_) => "Artist",
            Screen::Playlists => "Playlists",
            Screen::PlaylistDetail(_) => "Playlist",
            Screen::Queue => "Queue",
            Screen::Downloads => "Downloads",
            Screen::Settings => "Settings",
            Screen::Help => "Help",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            Screen::Home => "󰋜",
            Screen::Search => "󰥈",
            Screen::Library => "󰋭",
            Screen::Albums => "󰀥",
            Screen::AlbumDetail(_) => "󰀥",
            Screen::Artists => "󰠃",
            Screen::ArtistDetail(_) => "󰠃",
            Screen::Playlists => "󰋋",
            Screen::PlaylistDetail(_) => "󰋋",
            Screen::Queue => "󰙡",
            Screen::Downloads => "󰶅",
            Screen::Settings => "󰒓",
            Screen::Help => "󰋖",
        }
    }

    pub fn navigate_to(&self) -> Vec<Screen> {
        match self {
            Screen::AlbumDetail(_id) => vec![Screen::Albums, self.clone()],
            Screen::ArtistDetail(_id) => vec![Screen::Artists, self.clone()],
            Screen::PlaylistDetail(_id) => vec![Screen::Playlists, self.clone()],
            _ => vec![self.clone()],
        }
    }
}

// ─── Events ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum AppEvent {
    Tick,
    Key(crossterm::event::KeyEvent),
    Mouse(crossterm::event::MouseEvent),
    Resize(u16, u16),
    PlaybackUpdate(PlaybackState),
    TrackChanged(TrackId),
    PlaybackFinished,
    SearchComplete(SearchResults),
    ProviderChanged(ProviderId),
    Notification(String),
    Error(String),
}

// ─── Player Commands ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum PlayerCommand {
    Play(TrackId),
    PlayPause,
    Pause,
    Resume,
    Stop,
    Next,
    Previous,
    Seek(Duration),
    SeekForward(Duration),
    SeekBackward(Duration),
    SetVolume(f64),
    VolumeUp(f64),
    VolumeDown(f64),
    SetRepeat(RepeatMode),
    ToggleRepeat,
    SetShuffle(ShuffleMode),
    ToggleShuffle,
    AddToQueue(TrackId),
    RemoveFromQueue(usize),
    ClearQueue,
    MoveInQueue(usize, usize),
    PlayFromQueue(usize),
}

// ─── Helpers ──────────────────────────────────────────────────────────────

pub fn format_duration(d: Duration) -> String {
    let total_secs = d.as_secs();
    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;
    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, mins, secs)
    } else {
        format!("{}:{:02}", mins, secs)
    }
}

pub fn parse_duration(s: &str) -> Result<Duration, String> {
    let parts: Vec<&str> = s.split(':').collect();
    match parts.len() {
        3 => {
            let h: u64 = parts[0].parse().map_err(|_| "Invalid hours")?;
            let m: u64 = parts[1].parse().map_err(|_| "Invalid minutes")?;
            let s: u64 = parts[2].parse().map_err(|_| "Invalid seconds")?;
            Ok(Duration::from_secs(h * 3600 + m * 60 + s))
        }
        2 => {
            let m: u64 = parts[0].parse().map_err(|_| "Invalid minutes")?;
            let s: u64 = parts[1].parse().map_err(|_| "Invalid seconds")?;
            Ok(Duration::from_secs(m * 60 + s))
        }
        _ => Err("Invalid duration format".to_string()),
    }
}

// ─── Library ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Library {
    pub tracks: HashMap<TrackId, Track>,
    pub albums: HashMap<AlbumId, Album>,
    pub artists: HashMap<ArtistId, Artist>,
    pub playlists: HashMap<PlaylistId, Playlist>,
}

impl Default for Library {
    fn default() -> Self {
        Self {
            tracks: HashMap::new(),
            albums: HashMap::new(),
            artists: HashMap::new(),
            playlists: HashMap::new(),
        }
    }
}

impl Library {
    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }

    pub fn album_count(&self) -> usize {
        self.albums.len()
    }

    pub fn artist_count(&self) -> usize {
        self.artists.len()
    }

    pub fn playlist_count(&self) -> usize {
        self.playlists.len()
    }

    pub fn duration_total(&self) -> Duration {
        self.tracks
            .values()
            .map(|t| t.duration)
            .sum::<Duration>()
    }
}
