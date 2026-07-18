mod local;
mod mock;
mod registry;
mod traits;
mod youtube;

pub use local::LocalProvider;
pub use mock::MockProvider;
pub use registry::ProviderRegistry;
pub use traits::MusicProvider;
pub use youtube::YouTubeProvider;
