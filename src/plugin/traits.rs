use async_trait::async_trait;
use crate::errors::*;
use crate::types::*;
use crate::state::AppState;

#[async_trait]
pub trait Plugin: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn category(&self) -> PluginCategory;

    fn priority(&self) -> u8;

    async fn on_ready(&mut self, state: &AppState) -> Result<()>;
    async fn on_event(&mut self, event: &AppEvent, state: &mut AppState) -> Result<()>;
    async fn on_tick(&mut self, state: &mut AppState) -> Result<()>;
    async fn on_shutdown(&mut self) -> Result<()>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginCategory {
    Extension,
    Visualizer,
    Lyrics,
    Theme,
    Provider,
    Other,
}

#[async_trait]
pub trait PluginDefaults: Plugin {
    fn priority(&self) -> u8 {
        0
    }

    async fn on_ready(&mut self, _state: &AppState) -> Result<()> {
        Ok(())
    }

    async fn on_event(&mut self, _event: &AppEvent, _state: &mut AppState) -> Result<()> {
        Ok(())
    }

    async fn on_tick(&mut self, _state: &mut AppState) -> Result<()> {
        Ok(())
    }

    async fn on_shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}
