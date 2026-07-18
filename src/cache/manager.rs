use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::errors::*;

pub struct CacheManager {
    cache_dir: PathBuf,
    #[allow(dead_code)]
    max_size: u64,
    ttl: Duration,
}

impl CacheManager {
    pub fn new(cache_dir: PathBuf, max_size_mb: u64, ttl_hours: u64) -> Self {
        std::fs::create_dir_all(&cache_dir).ok();
        let covers_dir = cache_dir.join("covers");
        std::fs::create_dir_all(&covers_dir).ok();
        let metadata_dir = cache_dir.join("metadata");
        std::fs::create_dir_all(&metadata_dir).ok();
        Self {
            cache_dir,
            max_size: max_size_mb * 1024 * 1024,
            ttl: Duration::from_secs(ttl_hours * 3600),
        }
    }

    // ── Cover Art ────────────────────────────────────────────────────────

    pub fn get_cover_path(&self, url: &str) -> PathBuf {
        let ext = self.extension_for_url(url);
        let filename = format!("{}.{}", hash_key(url), ext);
        self.cache_dir.join("covers").join(filename)
    }

    pub fn has_cover(&self, url: &str) -> bool {
        self.get_cover_path(url).exists()
    }

    pub fn save_cover(&self, url: &str, data: &[u8]) -> Result<PathBuf> {
        let path = self.get_cover_path(url);
        std::fs::write(&path, data)
            .map_err(|e| SymphonyError::cache(format!("Failed to save cover: {e}")))?;
        Ok(path)
    }

    pub fn get_cover(&self, url: &str) -> Result<Option<Vec<u8>>> {
        let path = self.get_cover_path(url);
        if !path.exists() {
            return Ok(None);
        }
        if self.is_expired(&path) {
            std::fs::remove_file(&path).ok();
            return Ok(None);
        }
        let data = std::fs::read(&path)
            .map_err(|e| SymphonyError::cache(format!("Failed to read cover: {e}")))?;
        Ok(Some(data))
    }

    // ── Generic Cache ────────────────────────────────────────────────────

    pub fn get(&self, key: &str) -> Result<Option<String>> {
        let path = self.metadata_path(key);
        if !path.exists() {
            return Ok(None);
        }
        if self.is_expired(&path) {
            std::fs::remove_file(&path).ok();
            return Ok(None);
        }
        let data = std::fs::read_to_string(&path)
            .map_err(|e| SymphonyError::cache(format!("Failed to read cache: {e}")))?;
        Ok(Some(data))
    }

    pub fn set(&self, key: &str, value: &str) -> Result<()> {
        let path = self.metadata_path(key);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| SymphonyError::cache(format!("Failed to create cache dir: {e}")))?;
        }
        std::fs::write(&path, value)
            .map_err(|e| SymphonyError::cache(format!("Failed to write cache: {e}")))?;
        Ok(())
    }

    pub fn invalidate(&self, key: &str) -> Result<()> {
        let path = self.metadata_path(key);
        if path.exists() {
            std::fs::remove_file(&path)
                .map_err(|e| SymphonyError::cache(format!("Failed to invalidate cache: {e}")))?;
        }
        Ok(())
    }

    pub fn clear(&self) -> Result<()> {
        let dirs = [
            self.cache_dir.join("covers"),
            self.cache_dir.join("metadata"),
        ];
        for dir in &dirs {
            if dir.exists() {
                std::fs::remove_dir_all(dir)
                    .map_err(|e| SymphonyError::cache(format!("Failed to clear cache: {e}")))?;
                std::fs::create_dir_all(dir).map_err(|e| {
                    SymphonyError::cache(format!("Failed to recreate cache dir: {e}"))
                })?;
            }
        }
        Ok(())
    }

    // ── Maintenance ──────────────────────────────────────────────────────

    pub fn size(&self) -> Result<u64> {
        let mut total = 0u64;
        for entry in walk_dir(&self.cache_dir) {
            if let Ok(meta) = std::fs::metadata(&entry) {
                total += meta.len();
            }
        }
        Ok(total)
    }

    pub fn clean_expired(&self) -> Result<u64> {
        let mut removed = 0u64;
        for entry in walk_dir(&self.cache_dir) {
            if self.is_expired(&entry) {
                std::fs::remove_file(&entry).ok();
                removed += 1;
            }
        }
        Ok(removed)
    }

    pub fn clean_all(&self) -> Result<()> {
        self.clear()
    }

    // ── Helpers ──────────────────────────────────────────────────────────

    fn metadata_path(&self, key: &str) -> PathBuf {
        let filename = format!("{}.json", hash_key(key));
        self.cache_dir.join("metadata").join(filename)
    }

    fn is_expired(&self, path: &Path) -> bool {
        if let Ok(meta) = std::fs::metadata(path) {
            if let Ok(modified) = meta.modified() {
                if let Ok(elapsed) = SystemTime::now().duration_since(modified) {
                    return elapsed > self.ttl;
                }
            }
        }
        false
    }

    fn extension_for_url(&self, url: &str) -> &str {
        let url_lower = url.to_lowercase();
        for ext in &["webp", "png", "jpg", "jpeg", "gif", "bmp", "svg"] {
            if url_lower.ends_with(ext) {
                return if *ext == "jpeg" { "jpg" } else { ext };
            }
        }
        "jpg"
    }
}

fn hash_key(key: &str) -> String {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn walk_dir(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                files.push(path);
            } else if path.is_dir() {
                files.extend(walk_dir(&path));
            }
        }
    }
    files
}
