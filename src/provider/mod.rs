mod traits;
mod registry;
mod mock;
mod local;

pub use traits::MusicProvider;
pub use registry::ProviderRegistry;
pub use mock::MockProvider;
pub use local::LocalProvider;
