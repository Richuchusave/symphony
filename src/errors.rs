use thiserror::Error;

pub type Result<T> = std::result::Result<T, SymphonyError>;

#[derive(Error, Debug)]
pub enum SymphonyError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Provider error ({provider}): {message}")]
    Provider { provider: String, message: String },

    #[error("Playback error: {0}")]
    Playback(String),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("Plugin error: {0}")]
    Plugin(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unsupported: {0}")]
    Unsupported(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Theme error: {0}")]
    Theme(String),
}

impl SymphonyError {
    pub fn provider(provider: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Provider {
            provider: provider.into(),
            message: message.into(),
        }
    }

    pub fn playback(message: impl Into<String>) -> Self {
        Self::Playback(message.into())
    }

    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }

    pub fn plugin(message: impl Into<String>) -> Self {
        Self::Plugin(message.into())
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }

    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::Unsupported(message.into())
    }

    pub fn cache(message: impl Into<String>) -> Self {
        Self::Cache(message.into())
    }

    pub fn theme(message: impl Into<String>) -> Self {
        Self::Theme(message.into())
    }
}
