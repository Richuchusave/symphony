pub mod cache;
pub mod config;
pub mod db;
pub mod errors;
pub mod playback;
pub mod plugin;
pub mod provider;
pub mod state;
pub mod theme;
pub mod types;
pub mod ui;

mod app;

pub use app::Application;
pub use state::AppState;
