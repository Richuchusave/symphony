use async_trait::async_trait;
use fuzzy_matcher::skim::fuzzy_match;
use glob::glob;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use uuid::Uuid;

use crate::config::LocalProviderConfig;
use crate::errors::*;
use crate::provider::traits::MusicProvider;
use crate::types::*;

fn generate_id() -> String {
    Uuid::new_v4().to_string()
}

fn path_to_track_id(path: &Path) -> String {
    format!("local:{}", path.to_string_lossy())
}

fn path_to_album_id(album: &str, artist: &str) -> String {
    format!("local:album:{}/{}", artist, album)
}

fn path_to_artist_id(artist: &str) -> String {
    format!("local:artist:{}", artist)
}

fn read_metadata(path: &Path) -> Result<(String, String, String, Duration)> {
    let file_ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let supported = [
        "mp3", "flac", "wav", "ogg", "aac", "m4a", "opus",
    ];
    if !supported.contains(&file_ext.as_str()) {
        return Err(SymphonyError::unsupported(format!(
            "Unsupported file extension: {}",
            file_ext
        )));
    }

    let filename = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let (title, artist, album) = try_read_metadata_symphonia(path);

    let result = (
        title.unwrap_or_else(|| filename),
        artist.unwrap_or_else(|| "Unknown Artist".to_string()),
        album.unwrap_or_else(|| "Unknown Album".to_string()),
        Duration::from_secs(0),
    );

    Ok(result)
}

fn try_read_metadata_symphonia(path: &Path) -> (Option<String>, Option<String>, Option<String>) {
    let result = std::panic::catch_unwind(|| -> Option<(String, String, String)> {
        use symphonia::core::formats::FormatOptions;
        use symphonia::core::io::MediaSourceStream;
        use symphonia::core::meta::MetadataOptions;
        use symphonia::core::probe::Hint;

        let src = std::fs::File::open(path).ok()?;
        let mss = MediaSourceStream::new(Box::new(src), Default::default());
        let hint = Hint::new();
        let format_opts = FormatOptions::default();
        let meta_opts = MetadataOptions::default();

        let probe = symphonia::default::get_probe()
            .format(&hint, mss, &format_opts, &meta_opts)
            .ok()?;

        let mut reader = probe.format;
        let metadata = reader.metadata();
        let mut title = None;
        let mut artist = None;
        let mut album = None;

        if let Some(rev) = metadata.current() {
            for tag in rev.tags() {
                match tag.std_key {
                    Some(symphonia::core::meta::StandardTagKey::TrackTitle) => {
                        title = Some(tag.value.to_string());
                    }
                    Some(symphonia::core::meta::StandardTagKey::Artist) => {
                        artist = Some(tag.value.to_string());
                    }
                    Some(symphonia::core::meta::StandardTagKey::Album) => {
                        album = Some(tag.value.to_string());
                    }
                    _ => {}
                }
            }
        }

        let t = title.unwrap_or_default();
        let a = artist.unwrap_or_default();
        let al = album.unwrap_or_default();

        if t.is_empty() && a.is_empty() && al.is_empty() {
            None
        } else {
            Some((t, a, al))
        }
    });

    match result {
        Ok(Some((t, a, al))) => {
            let title = if t.is_empty() { None } else { Some(t) };
            let artist = if a.is_empty() { None } else { Some(a) };
            let album = if al.is_empty() { None } else { Some(al) };
            (title, artist, album)
        }
        _ => (None, None, None),
    }
}

pub struct LocalProvider {
    id: ProviderId,
    name: String,
    config: LocalProviderConfig,
    tracks: HashMap<TrackId, Track>,
    albums: HashMap<AlbumId, Album>,
    artists: HashMap<ArtistId, Artist>,
    playlists: HashMap<PlaylistId, Playlist>,
}

impl std::fmt::Debug for LocalProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalProvider")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("config", &self.config)
            .field("tracks", &self.tracks.len())
            .field("albums", &self.albums.len())
            .field("artists", &self.artists.len())
            .field("playlists", &self.playlists.len())
            .finish()
    }
}

impl LocalProvider {
    pub fn new(config: &LocalProviderConfig) -> Self {
        let id = generate_id();
        Self {
            id,
            name: "Local Files".to_string(),
            config: config.clone(),
            tracks: HashMap::new(),
            albums: HashMap::new(),
            artists: HashMap::new(),
            playlists: HashMap::new(),
        }
    }

    pub fn scan(&mut self) -> Result<()> {
        let id = self.id.clone();
        let mut tracks: HashMap<TrackId, Track> = HashMap::new();
        let mut albums_map: HashMap<AlbumId, Album> = HashMap::new();
        let mut artists_map: HashMap<ArtistId, Artist> = HashMap::new();

        for dir in &self.config.music_directories {
            let pattern = if self.config.scan_recursive {
                format!("{}/**/*", dir.to_string_lossy())
            } else {
                format!("{}/*", dir.to_string_lossy())
            };

            for entry in glob(&pattern).map_err(|e| {
                SymphonyError::playback(format!("Invalid glob pattern: {e}"))
            })? {
                match entry {
                    Ok(path) => {
                        if !path.is_file() {
                            continue;
                        }
                        let ext = path
                            .extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("")
                            .to_lowercase();
                        if !self.config.file_extensions.contains(&ext) {
                            continue;
                        }

                        match read_metadata(&path) {
                            Ok((title, artist_name, album_name, _duration)) => {
                                let track_id = path_to_track_id(&path);
                                let artist_id = path_to_artist_id(&artist_name);
                                let album_id = path_to_album_id(&album_name, &artist_name);

                                artists_map.entry(artist_id.clone()).or_insert(Artist {
                                    id: artist_id.clone(),
                                    name: artist_name.clone(),
                                    image_url: None,
                                    genres: Vec::new(),
                                    album_count: 0,
                                    provider: id.clone(),
                                });

                                let file_path = path.to_string_lossy().to_string();
                                let stream_url = format!("file://{}", file_path);

                                tracks.insert(
                                    track_id.clone(),
                                    Track {
                                        id: track_id.clone(),
                                        title,
                                        artist: artist_name.clone(),
                                        artist_id: artist_id.clone(),
                                        album: album_name.clone(),
                                        album_id: album_id.clone(),
                                        duration: Duration::from_secs(0),
                                        track_number: 0,
                                        disc_number: 1,
                                        genre: String::new(),
                                        year: 0,
                                        cover_url: None,
                                        stream_url: Some(stream_url),
                                        file_path: Some(file_path),
                                        provider: id.clone(),
                                        is_local: true,
                                    },
                                );
                            }
                            Err(_) => continue,
                        }
                    }
                    Err(_) => continue,
                }
            }
        }

        for track in tracks.values() {
            let album_entry = albums_map.entry(track.album_id.clone()).or_insert(Album {
                id: track.album_id.clone(),
                title: track.album.clone(),
                artist: track.artist.clone(),
                artist_id: track.artist_id.clone(),
                tracks: Vec::new(),
                cover_url: None,
                year: 0,
                track_count: 0,
                duration: Duration::ZERO,
                provider: id.clone(),
            });
            album_entry.tracks.push(track.id.clone());
            album_entry.track_count = album_entry.tracks.len() as u32;
            album_entry.duration += track.duration;

            if let Some(artist) = artists_map.get_mut(&track.artist_id) {
                artist.album_count = albums_map.len() as u32;
            }
        }

        self.tracks = tracks;
        self.albums = albums_map;
        self.artists = artists_map;
        self.playlists = HashMap::new();

        Ok(())
    }

    fn fuzzy_search<T, F, K>(items: &HashMap<K, T>, query: &str, f: F) -> Vec<T>
    where
        T: Clone,
        F: Fn(&T) -> &str,
        K: std::hash::Hash + Eq,
    {
        if query.is_empty() {
            return items.values().cloned().collect();
        }
        let q = query.to_lowercase();
        let mut scored: Vec<(i64, &T)> = items
            .values()
            .filter_map(|item| {
                let field = f(item).to_lowercase();
                let score = fuzzy_match(&field, &q)?;
                Some((score, item))
            })
            .collect();
        scored.sort_by(|a, b| b.0.cmp(&a.0));
        scored.into_iter().map(|(_, item)| item.clone()).collect()
    }
}

#[async_trait]
impl MusicProvider for LocalProvider {
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
            .get(id)
            .cloned()
            .ok_or_else(|| SymphonyError::not_found(format!("Track '{}' not found", id)))
    }

    async fn album(&self, id: &AlbumId) -> Result<Album> {
        self.albums
            .get(id)
            .cloned()
            .ok_or_else(|| SymphonyError::not_found(format!("Album '{}' not found", id)))
    }

    async fn artist(&self, id: &ArtistId) -> Result<Artist> {
        self.artists
            .get(id)
            .cloned()
            .ok_or_else(|| SymphonyError::not_found(format!("Artist '{}' not found", id)))
    }

    async fn artist_albums(&self, id: &ArtistId) -> Result<Vec<Album>> {
        let artist = self.artist(id).await?;
        Ok(self
            .albums
            .values()
            .filter(|a| a.artist_id == artist.id)
            .cloned()
            .collect())
    }

    async fn artist_top_tracks(&self, id: &ArtistId) -> Result<Vec<Track>> {
        let artist = self.artist(id).await?;
        let mut artist_tracks: Vec<&Track> = self
            .tracks
            .values()
            .filter(|t| t.artist_id == artist.id)
            .collect();
        artist_tracks.sort_by(|a, b| a.title.cmp(&b.title));
        Ok(artist_tracks.into_iter().take(5).cloned().collect())
    }

    async fn playlist(&self, id: &PlaylistId) -> Result<Playlist> {
        self.playlists
            .get(id)
            .cloned()
            .ok_or_else(|| SymphonyError::not_found(format!("Playlist '{}' not found", id)))
    }

    async fn resolve_stream_url(&self, track: &Track) -> Result<String> {
        track
            .stream_url
            .clone()
            .ok_or_else(|| SymphonyError::not_found(format!("No stream URL for track '{}'", track.id)))
    }

    async fn search_tracks(&self, query: &str, limit: u32, offset: u32) -> Result<Vec<Track>> {
        let results = Self::fuzzy_search(&self.tracks, query, |t| &t.title);
        let start = offset as usize;
        let end = (start + limit as usize).min(results.len());
        Ok(results[start..end].to_vec())
    }

    async fn search_albums(&self, query: &str, limit: u32, offset: u32) -> Result<Vec<Album>> {
        let results = Self::fuzzy_search(&self.albums, query, |a| &a.title);
        let start = offset as usize;
        let end = (start + limit as usize).min(results.len());
        Ok(results[start..end].to_vec())
    }

    async fn search_artists(&self, query: &str, limit: u32, offset: u32) -> Result<Vec<Artist>> {
        let results = Self::fuzzy_search(&self.artists, query, |a| &a.name);
        let start = offset as usize;
        let end = (start + limit as usize).min(results.len());
        Ok(results[start..end].to_vec())
    }

    async fn search_playlists(&self, _query: &str, limit: u32, offset: u32) -> Result<Vec<Playlist>> {
        let results: Vec<Playlist> = self.playlists.values().cloned().collect();
        let start = offset as usize;
        let end = (start + limit as usize).min(results.len());
        Ok(results[start..end].to_vec())
    }

    async fn browse_new_releases(&self, limit: u32) -> Result<Vec<Album>> {
        let mut albums: Vec<Album> = self.albums.values().cloned().collect();
        albums.sort_by(|a, b| b.year.cmp(&a.year));
        albums.truncate(limit as usize);
        Ok(albums)
    }

    async fn browse_featured_playlists(&self, _limit: u32) -> Result<Vec<Playlist>> {
        Ok(Vec::new())
    }

    async fn browse_categories(&self) -> Result<Vec<String>> {
        let mut genres: Vec<String> = self
            .tracks
            .values()
            .map(|t| t.genre.clone())
            .filter(|g| !g.is_empty())
            .collect();
        genres.sort();
        genres.dedup();
        Ok(genres)
    }
}
