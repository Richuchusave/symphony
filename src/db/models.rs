use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

use crate::errors::*;
use crate::types::*;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| SymphonyError::Database(rusqlite::Error::ToSqlConversionFailure(Box::new(e))))?;
        }
        let conn = Connection::open(path)?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.initialize_schema()?;
        Ok(db)
    }

    fn initialize_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS tracks (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                artist TEXT NOT NULL,
                artist_id TEXT NOT NULL,
                album TEXT NOT NULL,
                album_id TEXT NOT NULL,
                duration_secs INTEGER NOT NULL,
                track_number INTEGER NOT NULL,
                disc_number INTEGER NOT NULL,
                genre TEXT NOT NULL DEFAULT '',
                year INTEGER NOT NULL DEFAULT 0,
                cover_url TEXT,
                stream_url TEXT,
                file_path TEXT,
                provider TEXT NOT NULL,
                is_local INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS albums (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                artist TEXT NOT NULL,
                artist_id TEXT NOT NULL,
                track_ids TEXT NOT NULL DEFAULT '[]',
                cover_url TEXT,
                year INTEGER NOT NULL DEFAULT 0,
                track_count INTEGER NOT NULL DEFAULT 0,
                duration_secs INTEGER NOT NULL DEFAULT 0,
                provider TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS artists (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                image_url TEXT,
                genres TEXT NOT NULL DEFAULT '[]',
                album_count INTEGER NOT NULL DEFAULT 0,
                provider TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS playlists (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                cover_url TEXT,
                owner TEXT NOT NULL DEFAULT '',
                provider TEXT NOT NULL DEFAULT 'local',
                is_user_created INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS playlist_tracks (
                playlist_id TEXT NOT NULL,
                track_id TEXT NOT NULL,
                position INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (playlist_id, track_id),
                FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE,
                FOREIGN KEY (track_id) REFERENCES tracks(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS queue_state (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                track_ids TEXT NOT NULL DEFAULT '[]',
                current_index INTEGER
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS cache (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                expires_at INTEGER
            );
            ",
        )?;
        Ok(())
    }

    // ── Tracks ───────────────────────────────────────────────────────────

    pub fn save_track(&self, track: &Track) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO tracks (id, title, artist, artist_id, album, album_id, duration_secs, track_number, disc_number, genre, year, cover_url, stream_url, file_path, provider, is_local)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            params![
                track.id,
                track.title,
                track.artist,
                track.artist_id,
                track.album,
                track.album_id,
                track.duration.as_secs() as i64,
                track.track_number,
                track.disc_number,
                track.genre,
                track.year,
                track.cover_url,
                track.stream_url,
                track.file_path,
                track.provider,
                track.is_local as i32,
            ],
        )?;
        Ok(())
    }

    pub fn save_tracks(&self, tracks: &[Track]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let tx = conn.unchecked_transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT OR REPLACE INTO tracks (id, title, artist, artist_id, album, album_id, duration_secs, track_number, disc_number, genre, year, cover_url, stream_url, file_path, provider, is_local)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            )?;
            for track in tracks {
                stmt.execute(params![
                    track.id,
                    track.title,
                    track.artist,
                    track.artist_id,
                    track.album,
                    track.album_id,
                    track.duration.as_secs() as i64,
                    track.track_number,
                    track.disc_number,
                    track.genre,
                    track.year,
                    track.cover_url,
                    track.stream_url,
                    track.file_path,
                    track.provider,
                    track.is_local as i32,
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn get_track(&self, id: &TrackId) -> Result<Option<Track>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, artist, artist_id, album, album_id, duration_secs, track_number, disc_number, genre, year, cover_url, stream_url, file_path, provider, is_local
             FROM tracks WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], row_to_track)?;
        match rows.next() {
            Some(Ok(track)) => Ok(Some(track)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    pub fn get_all_tracks(&self) -> Result<Vec<Track>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, artist, artist_id, album, album_id, duration_secs, track_number, disc_number, genre, year, cover_url, stream_url, file_path, provider, is_local
             FROM tracks ORDER BY artist, album, disc_number, track_number",
        )?;
        let rows = stmt.query_map([], row_to_track)?;
        let mut tracks = Vec::new();
        for row in rows {
            tracks.push(row?);
        }
        Ok(tracks)
    }

    pub fn delete_track(&self, id: &TrackId) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM tracks WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn search_tracks(&self, query: &str) -> Result<Vec<Track>> {
        let pattern = format!("%{}%", query);
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, artist, artist_id, album, album_id, duration_secs, track_number, disc_number, genre, year, cover_url, stream_url, file_path, provider, is_local
             FROM tracks WHERE title LIKE ?1 OR artist LIKE ?1 OR album LIKE ?1
             ORDER BY artist, album, disc_number, track_number LIMIT 500",
        )?;
        let rows = stmt.query_map(params![pattern], row_to_track)?;
        let mut tracks = Vec::new();
        for row in rows {
            tracks.push(row?);
        }
        Ok(tracks)
    }

    // ── Albums ───────────────────────────────────────────────────────────

    pub fn save_album(&self, album: &Album) -> Result<()> {
        let track_ids = serde_json::to_string(&album.tracks)?;
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO albums (id, title, artist, artist_id, track_ids, cover_url, year, track_count, duration_secs, provider)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                album.id,
                album.title,
                album.artist,
                album.artist_id,
                track_ids,
                album.cover_url,
                album.year,
                album.track_count,
                album.duration.as_secs() as i64,
                album.provider,
            ],
        )?;
        Ok(())
    }

    pub fn get_album(&self, id: &AlbumId) -> Result<Option<Album>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, artist, artist_id, track_ids, cover_url, year, track_count, duration_secs, provider
             FROM albums WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], row_to_album)?;
        match rows.next() {
            Some(Ok(album)) => Ok(Some(album)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    pub fn get_all_albums(&self) -> Result<Vec<Album>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, artist, artist_id, track_ids, cover_url, year, track_count, duration_secs, provider
             FROM albums ORDER BY artist, year",
        )?;
        let rows = stmt.query_map([], row_to_album)?;
        let mut albums = Vec::new();
        for row in rows {
            albums.push(row?);
        }
        Ok(albums)
    }

    // ── Artists ──────────────────────────────────────────────────────────

    pub fn save_artist(&self, artist: &Artist) -> Result<()> {
        let genres = serde_json::to_string(&artist.genres)?;
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO artists (id, name, image_url, genres, album_count, provider)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                artist.id,
                artist.name,
                artist.image_url,
                genres,
                artist.album_count,
                artist.provider,
            ],
        )?;
        Ok(())
    }

    pub fn get_artist(&self, id: &ArtistId) -> Result<Option<Artist>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, image_url, genres, album_count, provider
             FROM artists WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], row_to_artist)?;
        match rows.next() {
            Some(Ok(artist)) => Ok(Some(artist)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    pub fn get_all_artists(&self) -> Result<Vec<Artist>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, image_url, genres, album_count, provider
             FROM artists ORDER BY name",
        )?;
        let rows = stmt.query_map([], row_to_artist)?;
        let mut artists = Vec::new();
        for row in rows {
            artists.push(row?);
        }
        Ok(artists)
    }

    // ── Playlists ────────────────────────────────────────────────────────

    pub fn save_playlist(&self, playlist: &Playlist) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO playlists (id, name, description, cover_url, owner, provider, is_user_created, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                playlist.id,
                playlist.name,
                playlist.description,
                playlist.cover_url,
                playlist.owner,
                playlist.provider,
                playlist.is_user_created as i32,
                playlist.created_at.to_rfc3339(),
                playlist.updated_at.to_rfc3339(),
            ],
        )?;
        conn.execute(
            "DELETE FROM playlist_tracks WHERE playlist_id = ?1",
            params![playlist.id],
        )?;
        {
            let mut stmt = conn.prepare(
                "INSERT INTO playlist_tracks (playlist_id, track_id, position) VALUES (?1, ?2, ?3)",
            )?;
            for (i, track_id) in playlist.tracks.iter().enumerate() {
                stmt.execute(params![playlist.id, track_id, i as i64])?;
            }
        }
        Ok(())
    }

    pub fn get_playlist(&self, id: &PlaylistId) -> Result<Option<Playlist>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, description, cover_url, owner, provider, is_user_created, created_at, updated_at
             FROM playlists WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            row_to_playlist(row, &conn)
        })?;
        match rows.next() {
            Some(Ok(playlist)) => Ok(Some(playlist)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    pub fn get_all_playlists(&self) -> Result<Vec<Playlist>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, description, cover_url, owner, provider, is_user_created, created_at, updated_at
             FROM playlists ORDER BY name",
        )?;
        let rows = stmt.query_map([], |row| {
            row_to_playlist(row, &conn)
        })?;
        let mut playlists = Vec::new();
        for row in rows {
            playlists.push(row?);
        }
        Ok(playlists)
    }

    pub fn delete_playlist(&self, id: &PlaylistId) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM playlist_tracks WHERE playlist_id = ?1", params![id])?;
        conn.execute("DELETE FROM playlists WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn add_track_to_playlist(
        &self,
        playlist_id: &PlaylistId,
        track_id: &TrackId,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let max_pos: i64 = conn.query_row(
            "SELECT COALESCE(MAX(position), -1) FROM playlist_tracks WHERE playlist_id = ?1",
            params![playlist_id],
            |row| row.get(0),
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO playlist_tracks (playlist_id, track_id, position) VALUES (?1, ?2, ?3)",
            params![playlist_id, track_id, max_pos + 1],
        )?;
        Ok(())
    }

    pub fn remove_track_from_playlist(
        &self,
        playlist_id: &PlaylistId,
        track_id: &TrackId,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM playlist_tracks WHERE playlist_id = ?1 AND track_id = ?2",
            params![playlist_id, track_id],
        )?;
        Ok(())
    }

    // ── Queue ────────────────────────────────────────────────────────────

    pub fn save_queue(
        &self,
        queue: &[TrackId],
        current_index: Option<usize>,
    ) -> Result<()> {
        let track_ids = serde_json::to_string(queue)?;
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM queue_state", [])?;
        conn.execute(
            "INSERT INTO queue_state (track_ids, current_index) VALUES (?1, ?2)",
            params![
                track_ids,
                current_index.map(|i| i as i64),
            ],
        )?;
        Ok(())
    }

    pub fn load_queue(&self) -> Result<(Vec<TrackId>, Option<usize>)> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT track_ids, current_index FROM queue_state ORDER BY id DESC LIMIT 1",
        )?;
        let result = stmt.query_row([], |row| {
            let json: String = row.get(0)?;
            let index: Option<i64> = row.get(1)?;
            Ok((json, index))
        });
        match result {
            Ok((json, index)) => {
                let tracks: Vec<TrackId> = serde_json::from_str(&json)?;
                Ok((tracks, index.map(|i| i as usize)))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok((Vec::new(), None)),
            Err(e) => Err(e.into()),
        }
    }

    // ── Settings ─────────────────────────────────────────────────────────

    pub fn save_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT value FROM settings WHERE key = ?1",
        )?;
        let mut rows = stmt.query_map(params![key], |row| row.get::<_, String>(0))?;
        match rows.next() {
            Some(Ok(value)) => Ok(Some(value)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    // ── Cache ────────────────────────────────────────────────────────────

    pub fn get_cache(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();
        let mut stmt = conn.prepare(
            "SELECT value FROM cache WHERE key = ?1 AND (expires_at IS NULL OR expires_at > ?2)",
        )?;
        let mut rows = stmt.query_map(params![key, now], |row| row.get::<_, String>(0))?;
        match rows.next() {
            Some(Ok(value)) => Ok(Some(value)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    pub fn set_cache(&self, key: &str, value: &str, ttl_seconds: u64) -> Result<()> {
        let expires_at = chrono::Utc::now().timestamp() + ttl_seconds as i64;
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO cache (key, value, expires_at) VALUES (?1, ?2, ?3)",
            params![key, value, expires_at],
        )?;
        Ok(())
    }

    pub fn clean_expired_cache(&self) -> Result<u64> {
        let now = chrono::Utc::now().timestamp();
        let conn = self.conn.lock().unwrap();
        let deleted = conn.execute(
            "DELETE FROM cache WHERE expires_at IS NOT NULL AND expires_at <= ?1",
            params![now],
        )?;
        Ok(deleted as u64)
    }
}

// ── Row Mappers ─────────────────────────────────────────────────────────

fn row_to_track(row: &rusqlite::Row) -> rusqlite::Result<Track> {
    Ok(Track {
        id: row.get(0)?,
        title: row.get(1)?,
        artist: row.get(2)?,
        artist_id: row.get(3)?,
        album: row.get(4)?,
        album_id: row.get(5)?,
        duration: Duration::from_secs(row.get::<_, i64>(6)? as u64),
        track_number: row.get::<_, i32>(7)? as u32,
        disc_number: row.get::<_, i32>(8)? as u32,
        genre: row.get(9)?,
        year: row.get(10)?,
        cover_url: row.get(11)?,
        stream_url: row.get(12)?,
        file_path: row.get(13)?,
        provider: row.get(14)?,
        is_local: row.get::<_, i32>(15)? != 0,
    })
}

fn row_to_album(row: &rusqlite::Row) -> rusqlite::Result<Album> {
    let track_ids_json: String = row.get(4)?;
    let tracks: Vec<TrackId> = serde_json::from_str(&track_ids_json)
        .unwrap_or_default();
    Ok(Album {
        id: row.get(0)?,
        title: row.get(1)?,
        artist: row.get(2)?,
        artist_id: row.get(3)?,
        tracks,
        cover_url: row.get(5)?,
        year: row.get(6)?,
        track_count: row.get::<_, i32>(7)? as u32,
        duration: Duration::from_secs(row.get::<_, i64>(8)? as u64),
        provider: row.get(9)?,
    })
}

fn row_to_artist(row: &rusqlite::Row) -> rusqlite::Result<Artist> {
    let genres_json: String = row.get(3)?;
    let genres: Vec<String> = serde_json::from_str(&genres_json)
        .unwrap_or_default();
    Ok(Artist {
        id: row.get(0)?,
        name: row.get(1)?,
        image_url: row.get(2)?,
        genres,
        album_count: row.get::<_, i32>(4)? as u32,
        provider: row.get(5)?,
    })
}

fn row_to_playlist(
    row: &rusqlite::Row,
    conn: &Connection,
) -> rusqlite::Result<Playlist> {
    let id: String = row.get(0)?;
    let mut track_stmt = conn.prepare(
        "SELECT track_id FROM playlist_tracks WHERE playlist_id = ?1 ORDER BY position",
    )?;
    let track_ids: Vec<TrackId> = track_stmt
        .query_map(params![id], |r| r.get(0))?
        .filter_map(|r| r.ok())
        .collect();
    let created_at_str: String = row.get(7)?;
    let updated_at_str: String = row.get(8)?;
    Ok(Playlist {
        id,
        name: row.get(1)?,
        description: row.get(2)?,
        tracks: track_ids,
        cover_url: row.get(3)?,
        owner: row.get(4)?,
        provider: row.get(5)?,
        is_user_created: row.get::<_, i32>(6)? != 0,
        created_at: created_at_str.parse().unwrap_or_else(|_| chrono::Utc::now()),
        updated_at: updated_at_str.parse().unwrap_or_else(|_| chrono::Utc::now()),
    })
}
