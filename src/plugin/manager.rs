use async_trait::async_trait;
use std::collections::HashMap;
use tracing;

use crate::plugin::traits::{Plugin, PluginCategory};
use crate::errors::*;
use crate::types::AppEvent;
use crate::state::AppState;

pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin + 'static>>,
    loaded: Vec<String>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            loaded: Vec::new(),
        }
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {

    pub fn register(&mut self, plugin: Box<dyn Plugin>) -> Result<()> {
        let id = plugin.id().to_string();
        if self.plugins.contains_key(&id) {
            return Err(SymphonyError::plugin(format!(
                "Plugin already registered: {}",
                id
            )));
        }
        self.plugins.insert(id.clone(), plugin);
        self.loaded.push(id);
        Ok(())
    }

    pub fn unregister(&mut self, id: &str) -> Option<Box<dyn Plugin>> {
        if let Some(idx) = self.loaded.iter().position(|p| p == id) {
            self.loaded.remove(idx);
        }
        self.plugins.remove(id)
    }

    pub fn get(&self, id: &str) -> Option<&dyn Plugin> {
        self.plugins.get(id).map(|p| p.as_ref())
    }

    pub fn get_mut<'a>(&'a mut self, id: &str) -> Option<&'a mut (dyn Plugin + 'static)> {
        self.plugins.get_mut(id).map(move |p| p.as_mut())
    }

    pub fn list(&self) -> Vec<&dyn Plugin> {
        self.loaded
            .iter()
            .filter_map(|id| self.plugins.get(id))
            .map(|p| p.as_ref())
            .collect()
    }

    pub fn list_by_category(&self, category: PluginCategory) -> Vec<&dyn Plugin> {
        self.loaded
            .iter()
            .filter_map(|id| self.plugins.get(id))
            .filter(|p| p.category() == category)
            .map(|p| p.as_ref())
            .collect()
    }

    pub fn loaded_count(&self) -> usize {
        self.loaded.len()
    }

    pub async fn initialize_all(&mut self, state: &AppState) -> Vec<Result<()>> {
        let mut results = Vec::new();
        let ids: Vec<String> = self.loaded.clone();
        for id in &ids {
            let result = if let Some(plugin) = self.plugins.get_mut(id) {
                plugin.on_ready(state).await
            } else {
                continue;
            };
            results.push(result);
        }
        results
    }

    pub async fn dispatch_event(
        &mut self,
        event: &AppEvent,
        state: &mut AppState,
    ) -> Vec<Result<()>> {
        let mut results = Vec::new();
        let ids: Vec<String> = self.loaded.clone();
        for id in &ids {
            let result = if let Some(plugin) = self.plugins.get_mut(id) {
                plugin.on_event(event, state).await
            } else {
                continue;
            };
            results.push(result);
        }
        results
    }

    pub async fn tick_all(&mut self, state: &mut AppState) -> Vec<Result<()>> {
        let mut results = Vec::new();
        let ids: Vec<String> = self.loaded.clone();
        for id in &ids {
            let result = if let Some(plugin) = self.plugins.get_mut(id) {
                plugin.on_tick(state).await
            } else {
                continue;
            };
            results.push(result);
        }
        results
    }

    pub async fn shutdown_all(&mut self) -> Vec<Result<()>> {
        let mut results = Vec::new();
        let ids: Vec<String> = self.loaded.clone();
        for id in &ids {
            let result = if let Some(plugin) = self.plugins.get_mut(id) {
                plugin.on_shutdown().await
            } else {
                continue;
            };
            results.push(result);
        }
        results
    }
}

// ── Example Plugins ──────────────────────────────────────────────────────

pub struct SimpleLoggerPlugin;

#[async_trait]
impl Plugin for SimpleLoggerPlugin {
    fn id(&self) -> &'static str {
        "simple-logger"
    }
    fn name(&self) -> &'static str {
        "Simple Logger"
    }
    fn description(&self) -> &'static str {
        "Logs plugin lifecycle events"
    }
    fn version(&self) -> &'static str {
        "0.1.0"
    }
    fn category(&self) -> PluginCategory {
        PluginCategory::Extension
    }
    fn priority(&self) -> u8 {
        0
    }

    async fn on_ready(&mut self, state: &AppState) -> Result<()> {
        tracing::info!(
            "Logger plugin ready | tracks={} albums={} artists={}",
            state.library.track_count(),
            state.library.album_count(),
            state.library.artist_count()
        );
        Ok(())
    }

    async fn on_event(&mut self, event: &AppEvent, _state: &mut AppState) -> Result<()> {
        tracing::debug!("Event: {:?}", event);
        Ok(())
    }

    async fn on_tick(&mut self, _state: &mut AppState) -> Result<()> {
        Ok(())
    }

    async fn on_shutdown(&mut self) -> Result<()> {
        tracing::info!("Logger plugin shutting down");
        Ok(())
    }
}

pub struct TickCounterPlugin {
    tick_count: u64,
}

impl TickCounterPlugin {
    pub fn new() -> Self {
        Self { tick_count: 0 }
    }
}

impl Default for TickCounterPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for TickCounterPlugin {
    fn id(&self) -> &'static str {
        "tick-counter"
    }
    fn name(&self) -> &'static str {
        "Tick Counter"
    }
    fn description(&self) -> &'static str {
        "Counts ticks for benchmarking"
    }
    fn version(&self) -> &'static str {
        "0.1.0"
    }
    fn category(&self) -> PluginCategory {
        PluginCategory::Other
    }
    fn priority(&self) -> u8 {
        255
    }

    async fn on_ready(&mut self, _state: &AppState) -> Result<()> {
        self.tick_count = 0;
        Ok(())
    }

    async fn on_event(&mut self, _event: &AppEvent, _state: &mut AppState) -> Result<()> {
        Ok(())
    }

    async fn on_tick(&mut self, _state: &mut AppState) -> Result<()> {
        self.tick_count += 1;
        if self.tick_count.is_multiple_of(1000) {
            tracing::debug!("Tick count: {}", self.tick_count);
        }
        Ok(())
    }

    async fn on_shutdown(&mut self) -> Result<()> {
        tracing::info!("Total ticks: {}", self.tick_count);
        Ok(())
    }
}
