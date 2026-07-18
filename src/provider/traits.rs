use std::fmt::Debug;

use async_trait::async_trait;

use crate::errors::*;
use crate::types::*;

#[async_trait]
pub trait MusicProvider: Send + Sync + Debug {
    fn id(&self) -> ProviderId;
    fn name(&self) -> &str;
    fn is_authenticated(&self) -> bool;

    async fn search(&self, query: &str, limit: u32, offset: u32) -> Result<SearchResults>;
    async fn track(&self, id: &TrackId) -> Result<Track>;
    async fn album(&self, id: &AlbumId) -> Result<Album>;
    async fn artist(&self, id: &ArtistId) -> Result<Artist>;
    async fn artist_albums(&self, id: &ArtistId) -> Result<Vec<Album>>;
    async fn artist_top_tracks(&self, id: &ArtistId) -> Result<Vec<Track>>;
    async fn playlist(&self, id: &PlaylistId) -> Result<Playlist>;
    async fn resolve_stream_url(&self, track: &Track) -> Result<String>;

    async fn search_tracks(&self, query: &str, limit: u32, offset: u32) -> Result<Vec<Track>>;
    async fn search_albums(&self, query: &str, limit: u32, offset: u32) -> Result<Vec<Album>>;
    async fn search_artists(&self, query: &str, limit: u32, offset: u32) -> Result<Vec<Artist>>;
    async fn search_playlists(&self, query: &str, limit: u32, offset: u32)
        -> Result<Vec<Playlist>>;

    async fn browse_new_releases(&self, limit: u32) -> Result<Vec<Album>>;
    async fn browse_featured_playlists(&self, limit: u32) -> Result<Vec<Playlist>>;
    async fn browse_categories(&self) -> Result<Vec<String>>;
}
