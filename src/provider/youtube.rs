use async_trait::async_trait;
use parking_lot::RwLock;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;

use crate::config::YouTubeMusicConfig;
use crate::errors::*;
use crate::provider::traits::MusicProvider;
use crate::types::*;

const PROVIDER_ID: &str = "youtube";
const MAX_SEARCH_RESULTS: u32 = 20;

#[derive(Debug, Deserialize)]
struct SearchResponse {
    #[serde(default)]
    entries: Vec<SearchEntry>,
}

#[derive(Debug, Deserialize)]
struct SearchEntry {
    id: String,
    title: String,
    #[serde(default)]
    uploader: Option<String>,
    #[serde(default)]
    channel: Option<String>,
    #[serde(default)]
    channel_id: Option<String>,
    #[serde(default)]
    duration: Option<f64>,
    #[serde(default)]
    thumbnails: Vec<Thumbnail>,
}

#[derive(Debug, Deserialize)]
struct Thumbnail {
    url: String,
    #[serde(default)]
    width: Option<u32>,
}

#[derive(Debug)]
pub struct YouTubeProvider {
    config: YouTubeMusicConfig,
    tracks: RwLock<HashMap<TrackId, Track>>,
    yt_dlp: PathBuf,
}

impl YouTubeProvider {
    pub fn new(config: YouTubeMusicConfig) -> Self {
        Self {
            config,
            tracks: RwLock::new(HashMap::new()),
            yt_dlp: find_yt_dlp(),
        }
    }

    fn entry_to_track(entry: SearchEntry) -> Track {
        let artist = entry
            .uploader
            .or(entry.channel)
            .unwrap_or_else(|| "YouTube".to_string());
        let artist_id = entry
            .channel_id
            .unwrap_or_else(|| format!("youtube:artist:{artist}"));
        let cover_url = entry
            .thumbnails
            .into_iter()
            .max_by_key(|thumbnail| thumbnail.width.unwrap_or_default())
            .map(|thumbnail| thumbnail.url);
        let duration = Duration::from_secs_f64(entry.duration.unwrap_or_default().max(0.0));
        let video_id = entry.id;

        Track {
            id: format!("youtube:{video_id}"),
            title: entry.title,
            artist,
            artist_id,
            album: "YouTube".to_string(),
            album_id: "youtube:album:singles".to_string(),
            duration,
            track_number: 1,
            disc_number: 1,
            genre: "stream".to_string(),
            year: 0,
            cover_url,
            stream_url: Some(format!("https://www.youtube.com/watch?v={video_id}")),
            file_path: None,
            provider: PROVIDER_ID.to_string(),
            is_local: false,
        }
    }

    async fn run_search(&self, query: &str, count: u32) -> Result<Vec<Track>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }

        let search = format!("ytsearch{count}:{}", query.trim());
        let mut command = Command::new(&self.yt_dlp);
        command
            .arg("--flat-playlist")
            .arg("--dump-single-json")
            .arg("--playlist-end")
            .arg(count.to_string())
            .arg("--no-warnings")
            .arg("--no-download")
            .arg("--socket-timeout")
            .arg("15")
            .arg(search)
            .stdin(Stdio::null())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped());

        if let Some(cookies_path) = self
            .config
            .cookies_path
            .as_ref()
            .filter(|path| path.exists())
        {
            command.arg("--cookies").arg(cookies_path);
        }

        let output = command.output().await.map_err(|error| {
            SymphonyError::provider(
                PROVIDER_ID,
                format!("Could not run {}: {error}", self.yt_dlp.display()),
            )
        })?;

        if !output.status.success() {
            let message = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(SymphonyError::provider(
                PROVIDER_ID,
                if message.is_empty() {
                    "YouTube search failed".to_string()
                } else {
                    message
                },
            ));
        }

        let response: SearchResponse = serde_json::from_slice(&output.stdout).map_err(|error| {
            SymphonyError::provider(PROVIDER_ID, format!("Invalid search response: {error}"))
        })?;
        let tracks: Vec<_> = response
            .entries
            .into_iter()
            .filter(|entry| !entry.id.is_empty() && !entry.title.is_empty())
            .map(Self::entry_to_track)
            .collect();

        {
            let mut cache = self.tracks.write();
            for track in &tracks {
                cache.insert(track.id.clone(), track.clone());
            }
        }

        Ok(tracks)
    }
}

fn find_yt_dlp() -> PathBuf {
    if let Some(path) = dirs::home_dir()
        .map(|home| home.join(".local/bin/yt-dlp"))
        .filter(|path| path.is_file())
    {
        path
    } else {
        Path::new("yt-dlp").to_path_buf()
    }
}

#[async_trait]
impl MusicProvider for YouTubeProvider {
    fn id(&self) -> ProviderId {
        PROVIDER_ID.to_string()
    }

    fn name(&self) -> &str {
        "YouTube"
    }

    fn is_authenticated(&self) -> bool {
        self.config.cookies_path.is_some() || self.config.use_oauth
    }

    async fn search(&self, query: &str, limit: u32, offset: u32) -> Result<SearchResults> {
        let tracks = self.search_tracks(query, limit, offset).await?;
        let total_results = tracks.len() as u32;
        Ok(SearchResults {
            tracks,
            albums: Vec::new(),
            artists: Vec::new(),
            playlists: Vec::new(),
            query: query.to_string(),
            total_results,
        })
    }

    async fn track(&self, id: &TrackId) -> Result<Track> {
        self.tracks
            .read()
            .get(id)
            .cloned()
            .ok_or_else(|| SymphonyError::not_found(format!("Track '{id}' not found")))
    }

    async fn album(&self, id: &AlbumId) -> Result<Album> {
        Err(SymphonyError::unsupported(format!(
            "YouTube album lookup is not implemented for '{id}'"
        )))
    }

    async fn artist(&self, id: &ArtistId) -> Result<Artist> {
        Err(SymphonyError::unsupported(format!(
            "YouTube artist lookup is not implemented for '{id}'"
        )))
    }

    async fn artist_albums(&self, _id: &ArtistId) -> Result<Vec<Album>> {
        Ok(Vec::new())
    }

    async fn artist_top_tracks(&self, _id: &ArtistId) -> Result<Vec<Track>> {
        Ok(Vec::new())
    }

    async fn playlist(&self, id: &PlaylistId) -> Result<Playlist> {
        Err(SymphonyError::unsupported(format!(
            "YouTube playlist lookup is not implemented for '{id}'"
        )))
    }

    async fn resolve_stream_url(&self, track: &Track) -> Result<String> {
        let video_id = track.id.strip_prefix("youtube:").ok_or_else(|| {
            SymphonyError::provider(PROVIDER_ID, format!("Invalid track id '{}'", track.id))
        })?;
        Ok(format!(
            "youtube://{video_id}?duration={}",
            track.duration.as_secs()
        ))
    }

    async fn search_tracks(&self, query: &str, limit: u32, offset: u32) -> Result<Vec<Track>> {
        let requested = limit.saturating_add(offset).clamp(1, MAX_SEARCH_RESULTS);
        let tracks = self.run_search(query, requested).await?;
        Ok(tracks
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect())
    }

    async fn search_albums(&self, _query: &str, _limit: u32, _offset: u32) -> Result<Vec<Album>> {
        Ok(Vec::new())
    }

    async fn search_artists(&self, _query: &str, _limit: u32, _offset: u32) -> Result<Vec<Artist>> {
        Ok(Vec::new())
    }

    async fn search_playlists(
        &self,
        _query: &str,
        _limit: u32,
        _offset: u32,
    ) -> Result<Vec<Playlist>> {
        Ok(Vec::new())
    }

    async fn browse_new_releases(&self, _limit: u32) -> Result<Vec<Album>> {
        Ok(Vec::new())
    }

    async fn browse_featured_playlists(&self, _limit: u32) -> Result<Vec<Playlist>> {
        Ok(Vec::new())
    }

    async fn browse_categories(&self) -> Result<Vec<String>> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_search_metadata_to_track() {
        let track = YouTubeProvider::entry_to_track(SearchEntry {
            id: "abc123".to_string(),
            title: "Example Song".to_string(),
            uploader: Some("Example Artist".to_string()),
            channel: None,
            channel_id: Some("artist123".to_string()),
            duration: Some(183.0),
            thumbnails: vec![Thumbnail {
                url: "https://example.test/cover.jpg".to_string(),
                width: Some(720),
            }],
        });

        assert_eq!(track.id, "youtube:abc123");
        assert_eq!(track.artist, "Example Artist");
        assert_eq!(track.duration, Duration::from_secs(183));
        assert_eq!(track.provider, PROVIDER_ID);
    }
}
