use std::collections::HashMap;

use crate::errors::*;
use crate::provider::traits::MusicProvider;
use crate::types::ProviderId;

pub struct ProviderRegistry {
    providers: HashMap<ProviderId, Box<dyn MusicProvider>>,
    active_provider: ProviderId,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            active_provider: String::new(),
        }
    }

    pub fn register(&mut self, provider: Box<dyn MusicProvider>) {
        let id = provider.id();
        self.active_provider = id.clone();
        self.providers.insert(id, provider);
    }

    pub fn unregister(&mut self, id: &ProviderId) -> Option<Box<dyn MusicProvider>> {
        let removed = self.providers.remove(id);
        if self.active_provider == *id {
            self.active_provider = self
                .providers
                .keys()
                .next()
                .cloned()
                .unwrap_or_default();
        }
        removed
    }

    pub fn get(&self, id: &ProviderId) -> Option<&dyn MusicProvider> {
        self.providers.get(id).map(|p| p.as_ref())
    }

    pub fn get_mut<'a>(&'a mut self, id: &ProviderId) -> Option<&'a mut (dyn MusicProvider + 'a)> {
        self.providers.get_mut(id).map(|p| p.as_mut())
    }

    pub fn active(&self) -> &dyn MusicProvider {
        self.providers
            .get(&self.active_provider)
            .map(|p| p.as_ref())
            .expect("No active provider registered")
    }

    pub fn active_mut(&mut self) -> &mut dyn MusicProvider {
        self.providers
            .get_mut(&self.active_provider)
            .map(|p| p.as_mut())
            .expect("No active provider registered")
    }

    pub fn set_active(&mut self, id: &ProviderId) -> Result<()> {
        if self.providers.contains_key(id) {
            self.active_provider = id.clone();
            Ok(())
        } else {
            Err(SymphonyError::not_found(format!(
                "Provider '{}' not found",
                id
            )))
        }
    }

    pub fn list(&self) -> Vec<&dyn MusicProvider> {
        self.providers.values().map(|p| p.as_ref()).collect()
    }

    pub fn list_ids(&self) -> Vec<ProviderId> {
        self.providers.keys().cloned().collect()
    }

    pub fn has_provider(&self, id: &ProviderId) -> bool {
        self.providers.contains_key(id)
    }

    pub fn len(&self) -> usize {
        self.providers.len()
    }
}
