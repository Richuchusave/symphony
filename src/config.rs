use crate::errors::{Result, SymphonyError};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub playback: PlaybackConfig,
    pub ui: UIConfig,
    pub cache: CacheConfig,
    pub providers: ProviderConfig,
    pub keybindings: KeybindingConfig,
    pub theme: ThemeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub data_dir: PathBuf,
    pub config_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub log_level: String,
    pub check_updates: bool,
    pub default_provider: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from(".symphony"))
            .join("symphony");
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from(".config"))
            .join("symphony");
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from(".cache"))
            .join("symphony");

        Self {
            data_dir,
            config_dir,
            cache_dir,
            log_level: "info".to_string(),
            check_updates: true,
            default_provider: "mock".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackConfig {
    pub volume: f64,
    pub default_volume: f64,
    pub volume_step: f64,
    pub seek_step_seconds: u64,
    pub crossfade: bool,
    pub crossfade_duration_seconds: u64,
    pub gapless: bool,
    pub replay_gain: bool,
    pub prebuffer_seconds: u64,
    pub audio_device: Option<String>,
}

impl Default for PlaybackConfig {
    fn default() -> Self {
        Self {
            volume: 0.8,
            default_volume: 0.8,
            volume_step: 0.05,
            seek_step_seconds: 5,
            crossfade: false,
            crossfade_duration_seconds: 3,
            gapless: true,
            replay_gain: false,
            prebuffer_seconds: 5,
            audio_device: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    pub show_clock: bool,
    pub show_cover_art: bool,
    pub cover_art_size: u16,
    pub sidebar_width: u16,
    pub progress_bar_char: String,
    pub progress_bar_fill: String,
    pub show_status_bar: bool,
    pub scroll_offset: usize,
    pub animation_fps: u8,
    pub mouse_support: bool,
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            show_clock: true,
            show_cover_art: true,
            cover_art_size: 8,
            sidebar_width: 22,
            progress_bar_char: "─".to_string(),
            progress_bar_fill: "━".to_string(),
            show_status_bar: true,
            scroll_offset: 5,
            animation_fps: 30,
            mouse_support: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub max_size_mb: u64,
    pub ttl_hours: u64,
    pub cache_metadata: bool,
    pub cache_cover_art: bool,
    pub cache_stream_urls: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size_mb: 500,
            ttl_hours: 24,
            cache_metadata: true,
            cache_cover_art: true,
            cache_stream_urls: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub enabled_providers: Vec<String>,
    pub local: LocalProviderConfig,
    pub spotify: Option<SpotifyConfig>,
    pub youtube_music: Option<YouTubeMusicConfig>,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            enabled_providers: vec!["mock".to_string(), "local".to_string()],
            local: LocalProviderConfig::default(),
            spotify: None,
            youtube_music: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalProviderConfig {
    pub music_directories: Vec<PathBuf>,
    pub file_extensions: Vec<String>,
    pub scan_recursive: bool,
    pub follow_symlinks: bool,
    pub watch_for_changes: bool,
}

impl Default for LocalProviderConfig {
    fn default() -> Self {
        let music_dir = dirs::audio_dir()
            .or_else(|| dirs::home_dir().map(|h| h.join("Music")));

        Self {
            music_directories: music_dir.map(|d| vec![d]).unwrap_or_default(),
            file_extensions: vec![
                "mp3".to_string(),
                "flac".to_string(),
                "wav".to_string(),
                "ogg".to_string(),
                "aac".to_string(),
                "m4a".to_string(),
                "opus".to_string(),
            ],
            scan_recursive: true,
            follow_symlinks: false,
            watch_for_changes: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub cache_token: bool,
}

impl Default for SpotifyConfig {
    fn default() -> Self {
        Self {
            client_id: String::new(),
            client_secret: String::new(),
            redirect_uri: "http://localhost:8888/callback".to_string(),
            cache_token: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeMusicConfig {
    pub cookies_path: Option<PathBuf>,
    pub use_oauth: bool,
    pub stream_quality: String,
    pub download_limit_mbps: u32,
}

impl Default for YouTubeMusicConfig {
    fn default() -> Self {
        Self {
            cookies_path: None,
            use_oauth: false,
            stream_quality: "high".to_string(),
            download_limit_mbps: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingConfig {
    pub keys: std::collections::HashMap<String, String>,
}

impl Default for KeybindingConfig {
    fn default() -> Self {
        let mut keys = std::collections::HashMap::new();
        keys.insert("play_pause".to_string(), "space".to_string());
        keys.insert("stop".to_string(), "s".to_string());
        keys.insert("next".to_string(), "l".to_string());
        keys.insert("previous".to_string(), "h".to_string());
        keys.insert("volume_up".to_string(), "up".to_string());
        keys.insert("volume_down".to_string(), "down".to_string());
        keys.insert("seek_forward".to_string(), "right".to_string());
        keys.insert("seek_backward".to_string(), "left".to_string());
        keys.insert("search".to_string(), "ctrl+k".to_string());
        keys.insert("command_palette".to_string(), "ctrl+p".to_string());
        keys.insert("help".to_string(), "?".to_string());
        keys.insert("quit".to_string(), "q".to_string());
        keys.insert("enter".to_string(), "enter".to_string());
        keys.insert("escape".to_string(), "esc".to_string());
        keys.insert("delete".to_string(), "delete".to_string());
        keys.insert("toggle_sidebar".to_string(), "ctrl+b".to_string());
        keys.insert("focus_search".to_string(), "/".to_string());
        keys.insert("queue".to_string(), "ctrl+q".to_string());
        keys.insert("save_playlist".to_string(), "ctrl+s".to_string());
        keys.insert("toggle_shuffle".to_string(), "z".to_string());
        keys.insert("toggle_repeat".to_string(), "x".to_string());
        Self { keys }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub name: String,
    pub custom_colors: Option<std::collections::HashMap<String, String>>,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            custom_colors: None,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config = Self::default();
        let config_path = config.general.config_dir.join("config.toml");

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .map_err(|e| SymphonyError::Config(format!("Failed to read config: {e}")))?;
            let parsed: Config = toml::from_str(&content)?;
            Ok(parsed)
        } else {
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = self.general.config_dir.join("config.toml");
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| SymphonyError::Config(format!("Failed to create config dir: {e}")))?;
        }
        let content = toml::to_string_pretty(&self)
            .map_err(|e| SymphonyError::Config(format!("Failed to serialize config: {e}")))?;
        std::fs::write(&config_path, content)
            .map_err(|e| SymphonyError::Config(format!("Failed to write config: {e}")))?;
        Ok(())
    }

    pub fn ensure_dirs(&self) -> Result<()> {
        for dir in &[&self.general.data_dir, &self.general.config_dir, &self.general.cache_dir] {
            std::fs::create_dir_all(dir)
                .map_err(|e| SymphonyError::Config(format!("Failed to create directory {dir:?}: {e}")))?;
        }
        Ok(())
    }
}
