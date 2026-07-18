use async_trait::async_trait;
use chrono::Utc;
use std::time::Duration;

use crate::errors::*;
use crate::provider::traits::MusicProvider;
use crate::types::*;

fn track_duration(album_index: usize, track_index: usize) -> Duration {
    Duration::from_secs(180 + ((album_index * 37 + track_index * 19) % 121) as u64)
}

fn make_cover_url(seed: &str) -> String {
    format!("https://picsum.photos/seed/{seed}/300/300")
}

const MOCK_ARTISTS: &[(&str, &[&str])] = &[
    ("Neon Cascade", &["electronic", "synthwave"]),
    ("Violet Mosaic", &["indie", "dream pop"]),
    ("Iron Horizon", &["rock", "alternative"]),
    ("Cosmic Drift", &["ambient", "electronic"]),
    ("The Velvet Echoes", &["jazz", "soul"]),
    ("Lunar Tides", &["lo-fi", "chillhop"]),
];

const MOCK_ALBUMS: &[(&str, usize, &str)] = &[
    ("Digital Twilight", 0, "electronic"),
    ("Neon Nights", 0, "synthwave"),
    ("Prism", 1, "indie"),
    ("Velvet Underground", 1, "dream pop"),
    ("Steel & Thunder", 2, "rock"),
    ("Broken Horizons", 2, "alternative"),
    ("Nebula", 3, "ambient"),
    ("Starfall", 3, "electronic"),
    ("Midnight Sessions", 4, "jazz"),
    ("Soul Revival", 4, "soul"),
    ("Ocean Waves", 5, "lo-fi"),
    ("Chill Tides", 5, "chillhop"),
];

#[derive(Debug)]
pub struct MockProvider {
    id: ProviderId,
    name: String,
    artists: Vec<Artist>,
    albums: Vec<Album>,
    tracks: Vec<Track>,
    playlists: Vec<Playlist>,
}

impl Default for MockProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl MockProvider {
    pub fn new() -> Self {
        let id = "mock".to_string();
        let provider_id = id.clone();
        let name = "Mock Provider".to_string();

        let artists: Vec<Artist> = MOCK_ARTISTS
            .iter()
            .enumerate()
            .map(|(i, (name, genres))| Artist {
                id: format!("mock:artist:{i}"),
                name: name.to_string(),
                image_url: Some(make_cover_url(name)),
                genres: genres.iter().map(|g| g.to_string()).collect(),
                album_count: MOCK_ALBUMS.iter().filter(|(_, ai, _)| *ai == i).count() as u32,
                provider: provider_id.clone(),
            })
            .collect();

        let mut tracks = Vec::new();
        let mut albums = Vec::new();

        for (album_idx, (album_title, artist_idx, _genre)) in MOCK_ALBUMS.iter().enumerate() {
            let artist = &artists[*artist_idx];
            let album_id = format!("mock:album:{album_idx}");
            let track_count = 6;
            let mut album_tracks = Vec::new();

            for t in 0..track_count {
                let track_id = format!("mock:track:{album_idx}:{t}");
                let duration = track_duration(album_idx, t);
                album_tracks.push(track_id.clone());
                tracks.push(Track {
                    id: track_id,
                    title: format!("{} — Track {}", album_title, t + 1),
                    artist: artist.name.clone(),
                    artist_id: artist.id.clone(),
                    album: album_title.to_string(),
                    album_id: album_id.clone(),
                    duration,
                    track_number: (t + 1) as u32,
                    disc_number: 1,
                    genre: MOCK_ALBUMS[album_idx].2.to_string(),
                    year: 2020 + (album_idx as i32 % 5),
                    cover_url: Some(make_cover_url(&format!("{album_title}-{t}"))),
                    stream_url: Some(format!("https://mock.stream/{}/{}", album_title, t + 1)),
                    file_path: None,
                    provider: provider_id.clone(),
                    is_local: false,
                });
            }

            let album_duration: Duration = tracks
                .iter()
                .rev()
                .take(track_count)
                .map(|t| t.duration)
                .sum();

            albums.push(Album {
                id: album_id,
                title: album_title.to_string(),
                artist: artist.name.clone(),
                artist_id: artist.id.clone(),
                tracks: album_tracks,
                cover_url: Some(make_cover_url(album_title)),
                year: 2020 + (album_idx as i32 % 5),
                track_count: track_count as u32,
                duration: album_duration,
                provider: provider_id.clone(),
            });
        }

        let playlists = vec![
            Playlist {
                id: "mock:playlist:electronic-essentials".to_string(),
                name: "Electronic Essentials".to_string(),
                description: "Best of electronic and synthwave".to_string(),
                tracks: tracks
                    .iter()
                    .filter(|t| t.genre == "electronic" || t.genre == "synthwave")
                    .take(10)
                    .map(|t| t.id.clone())
                    .collect(),
                cover_url: Some(make_cover_url("electronic-essentials")),
                owner: "mock-user".to_string(),
                provider: provider_id.clone(),
                is_user_created: false,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            Playlist {
                id: "mock:playlist:rock-anthems".to_string(),
                name: "Rock Anthems".to_string(),
                description: "Rock and alternative hits".to_string(),
                tracks: tracks
                    .iter()
                    .filter(|t| t.genre == "rock" || t.genre == "alternative")
                    .take(10)
                    .map(|t| t.id.clone())
                    .collect(),
                cover_url: Some(make_cover_url("rock-anthems")),
                owner: "mock-user".to_string(),
                provider: provider_id.clone(),
                is_user_created: false,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            Playlist {
                id: "mock:playlist:chill-vibes".to_string(),
                name: "Chill Vibes".to_string(),
                description: "Ambient, lo-fi, and chillhop".to_string(),
                tracks: tracks
                    .iter()
                    .filter(|t| t.genre == "ambient" || t.genre == "lo-fi" || t.genre == "chillhop")
                    .take(10)
                    .map(|t| t.id.clone())
                    .collect(),
                cover_url: Some(make_cover_url("chill-vibes")),
                owner: "mock-user".to_string(),
                provider: provider_id.clone(),
                is_user_created: false,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        ];

        Self {
            id,
            name,
            artists,
            albums,
            tracks,
            playlists,
        }
    }

    fn search_filter<T, F>(items: &[T], query: &str, f: F) -> Vec<T>
    where
        T: Clone,
        F: Fn(&T) -> &str,
    {
        if query.is_empty() {
            return items.to_vec();
        }
        let q = query.to_lowercase();
        items
            .iter()
            .filter(|item| f(item).to_lowercase().contains(&q))
            .cloned()
            .collect()
    }

    fn page<T>(items: Vec<T>, limit: u32, offset: u32) -> Vec<T> {
        items
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect()
    }
}

#[async_trait]
impl MusicProvider for MockProvider {
    fn id(&self) -> ProviderId {
        self.id.clone()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_authenticated(&self) -> bool {
        true
    }

    async fn search(&self, query: &str, limit: u32, offset: u32) -> Result<SearchResults> {
        let tracks = self.search_tracks(query, limit, offset).await?;
        let albums = self.search_albums(query, limit, offset).await?;
        let artists = self.search_artists(query, limit, offset).await?;
        let playlists = self.search_playlists(query, limit, offset).await?;
        let total = (tracks.len() + albums.len() + artists.len() + playlists.len()) as u32;

        Ok(SearchResults {
            tracks,
            albums,
            artists,
            playlists,
            query: query.to_string(),
            total_results: total,
        })
    }

    async fn track(&self, id: &TrackId) -> Result<Track> {
        self.tracks
            .iter()
            .find(|t| t.id == *id)
            .cloned()
            .ok_or_else(|| SymphonyError::not_found(format!("Track '{id}' not found")))
    }

    async fn album(&self, id: &AlbumId) -> Result<Album> {
        self.albums
            .iter()
            .find(|a| a.id == *id)
            .cloned()
            .ok_or_else(|| SymphonyError::not_found(format!("Album '{id}' not found")))
    }

    async fn artist(&self, id: &ArtistId) -> Result<Artist> {
        self.artists
            .iter()
            .find(|a| a.id == *id)
            .cloned()
            .ok_or_else(|| SymphonyError::not_found(format!("Artist '{id}' not found")))
    }

    async fn artist_albums(&self, id: &ArtistId) -> Result<Vec<Album>> {
        let artist = self.artist(id).await?;
        Ok(self
            .albums
            .iter()
            .filter(|a| a.artist_id == artist.id)
            .cloned()
            .collect())
    }

    async fn artist_top_tracks(&self, id: &ArtistId) -> Result<Vec<Track>> {
        let artist = self.artist(id).await?;
        Ok(self
            .tracks
            .iter()
            .filter(|t| t.artist_id == artist.id)
            .take(5)
            .cloned()
            .collect())
    }

    async fn playlist(&self, id: &PlaylistId) -> Result<Playlist> {
        self.playlists
            .iter()
            .find(|p| p.id == *id)
            .cloned()
            .ok_or_else(|| SymphonyError::not_found(format!("Playlist '{id}' not found")))
    }

    async fn resolve_stream_url(&self, track: &Track) -> Result<String> {
        Ok(track
            .stream_url
            .clone()
            .unwrap_or_else(|| "https://mock.stream/default".to_string()))
    }

    async fn search_tracks(&self, query: &str, limit: u32, offset: u32) -> Result<Vec<Track>> {
        let results = Self::search_filter(&self.tracks, query, |t| &t.title);
        Ok(Self::page(results, limit, offset))
    }

    async fn search_albums(&self, query: &str, limit: u32, offset: u32) -> Result<Vec<Album>> {
        let results = Self::search_filter(&self.albums, query, |a| &a.title);
        Ok(Self::page(results, limit, offset))
    }

    async fn search_artists(&self, query: &str, limit: u32, offset: u32) -> Result<Vec<Artist>> {
        let results = Self::search_filter(&self.artists, query, |a| &a.name);
        Ok(Self::page(results, limit, offset))
    }

    async fn search_playlists(
        &self,
        query: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<Playlist>> {
        let results = Self::search_filter(&self.playlists, query, |p| &p.name);
        Ok(Self::page(results, limit, offset))
    }

    async fn browse_new_releases(&self, limit: u32) -> Result<Vec<Album>> {
        let count = limit as usize;
        Ok(self.albums.iter().rev().take(count).cloned().collect())
    }

    async fn browse_featured_playlists(&self, limit: u32) -> Result<Vec<Playlist>> {
        let count = limit as usize;
        Ok(self.playlists.iter().take(count).cloned().collect())
    }

    async fn browse_categories(&self) -> Result<Vec<String>> {
        let mut categories: Vec<String> = self.tracks.iter().map(|t| t.genre.clone()).collect();
        categories.sort();
        categories.dedup();
        Ok(categories)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn catalog_ids_and_content_are_stable() {
        let first = MockProvider::new();
        let second = MockProvider::new();

        let first_tracks = first.search_tracks("", 100, 0).await.unwrap();
        let second_tracks = second.search_tracks("", 100, 0).await.unwrap();

        assert_eq!(first.id(), "mock");
        assert_eq!(first_tracks, second_tracks);
        assert_eq!(first_tracks.len(), MOCK_ALBUMS.len() * 6);
    }

    #[tokio::test]
    async fn pagination_past_the_end_returns_an_empty_page() {
        let provider = MockProvider::new();

        let page = provider.search_tracks("", 10, 10_000).await.unwrap();

        assert!(page.is_empty());
    }
}
