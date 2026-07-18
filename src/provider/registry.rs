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
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderRegistry {
    pub fn register(&mut self, provider: Box<dyn MusicProvider>) {
        let id = provider.id();
        if self.active_provider.is_empty() {
            self.active_provider = id.clone();
        }
        self.providers.insert(id, provider);
    }

    pub fn unregister(&mut self, id: &ProviderId) -> Option<Box<dyn MusicProvider>> {
        let removed = self.providers.remove(id);
        if self.active_provider == *id {
            self.active_provider = self.providers.keys().next().cloned().unwrap_or_default();
        }
        removed
    }

    pub fn get(&self, id: &ProviderId) -> Option<&dyn MusicProvider> {
        self.providers.get(id).map(|p| p.as_ref())
    }

    pub fn get_mut<'a>(
        &'a mut self,
        id: &ProviderId,
    ) -> Option<&'a mut (dyn MusicProvider + 'static)> {
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
                "Provider '{id}' not found"
            )))
        }
    }

    pub fn list(&self) -> Vec<&dyn MusicProvider> {
        self.providers.values().map(|p| p.as_ref()).collect()
    }

    pub fn list_ids(&self) -> Vec<ProviderId> {
        let mut ids: Vec<_> = self.providers.keys().cloned().collect();
        ids.sort();
        ids
    }

    pub fn has_provider(&self, id: &ProviderId) -> bool {
        self.providers.contains_key(id)
    }

    pub fn len(&self) -> usize {
        self.providers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::MockProvider;

    #[test]
    fn registration_keeps_the_first_provider_active() {
        let mut registry = ProviderRegistry::new();
        registry.register(Box::new(MockProvider::new()));

        assert_eq!(registry.active().id(), "mock");
        assert_eq!(registry.list_ids(), ["mock"]);
    }

    #[test]
    fn selecting_a_missing_provider_returns_an_error() {
        let mut registry = ProviderRegistry::new();
        registry.register(Box::new(MockProvider::new()));

        assert!(registry.set_active(&"missing".to_string()).is_err());
        assert_eq!(registry.active().id(), "mock");
    }
}
