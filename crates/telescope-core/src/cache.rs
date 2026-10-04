//! TTL key/value cache for API responses, kept in memory and persisted to a
//! single JSON file. Writes are batched: `set` only marks the cache dirty and
//! `flush` writes it out, so a large scan doesn't rewrite the file per pilot.
//! The file is read on a background thread; the first access waits for it.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use log::{info, warn};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
struct Entry {
    value: serde_json::Value,
    expires_at: u64,
}

#[derive(Default)]
struct Inner {
    entries: HashMap<String, Entry>,
    dirty: bool,
}

#[derive(Clone)]
pub struct Cache {
    path: Option<PathBuf>,
    inner: Arc<OnceLock<RwLock<Inner>>>,
}

fn load(path: &Path) -> Inner {
    let now = now_secs();
    let entries = std::fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<HashMap<String, Entry>>(&bytes).ok())
        .map(|mut entries| {
            entries.retain(|_, entry| entry.expires_at > now);
            entries
        })
        .unwrap_or_default();
    info!(
        "Loaded {} cache entries from {}",
        entries.len(),
        path.display()
    );
    Inner {
        entries,
        dirty: false,
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

impl Cache {
    pub fn open(path: &Path) -> Self {
        let inner: Arc<OnceLock<RwLock<Inner>>> = Arc::default();
        let (loading, file) = (inner.clone(), path.to_path_buf());
        std::thread::Builder::new()
            .name("cache-load".into())
            .spawn(move || {
                let _ = loading.set(RwLock::new(load(&file)));
            })
            .expect("failed to start cache loader");
        Self {
            path: Some(path.to_path_buf()),
            inner,
        }
    }

    pub fn in_memory() -> Self {
        let inner: Arc<OnceLock<RwLock<Inner>>> = Arc::default();
        let _ = inner.set(RwLock::default());
        Self { path: None, inner }
    }

    fn inner(&self) -> &RwLock<Inner> {
        self.inner.wait()
    }

    pub fn get(&self, key: &str) -> Option<serde_json::Value> {
        let inner = self.inner().read().ok()?;
        let entry = inner.entries.get(key)?;
        (entry.expires_at > now_secs()).then(|| entry.value.clone())
    }

    pub fn set(&self, key: &str, value: serde_json::Value, ttl_secs: u64) {
        if let Ok(mut inner) = self.inner().write() {
            inner.entries.insert(
                key.to_string(),
                Entry {
                    value,
                    expires_at: now_secs() + ttl_secs,
                },
            );
            inner.dirty = true;
        }
    }

    pub fn clear(&self) -> Result<(), String> {
        {
            let mut inner = self.inner().write().map_err(|e| e.to_string())?;
            inner.entries.clear();
            inner.dirty = true;
        }
        self.flush()
    }

    /// Prunes expired entries and writes the cache to disk if it changed.
    pub fn flush(&self) -> Result<(), String> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        let bytes = {
            let mut inner = self.inner().write().map_err(|e| e.to_string())?;
            if !inner.dirty {
                return Ok(());
            }
            let now = now_secs();
            inner.entries.retain(|_, entry| entry.expires_at > now);
            inner.dirty = false;
            serde_json::to_vec(&inner.entries).map_err(|e| e.to_string())?
        };
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, bytes)
            .and_then(|()| std::fs::rename(&tmp, path))
            .map_err(|e| {
                warn!("Failed to write cache {}: {}", path.display(), e);
                e.to_string()
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_returns_stored_value() {
        let cache = Cache::in_memory();
        cache.set("a", serde_json::json!(1), 60);
        assert_eq!(cache.get("a"), Some(serde_json::json!(1)));
        assert_eq!(cache.get("b"), None);
    }

    #[test]
    fn expired_entries_are_not_returned() {
        let cache = Cache::in_memory();
        cache.set("a", serde_json::json!(1), 0);
        assert_eq!(cache.get("a"), None);
    }

    #[test]
    fn flush_round_trips_through_disk() {
        let dir = std::env::temp_dir().join(format!("telescope-cache-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("cache.json");

        let cache = Cache::open(&path);
        cache.set("keep", serde_json::json!("x"), 60);
        cache.set("drop", serde_json::json!("y"), 0);
        cache.flush().unwrap();

        let reopened = Cache::open(&path);
        assert_eq!(reopened.get("keep"), Some(serde_json::json!("x")));
        assert_eq!(reopened.get("drop"), None);

        reopened.clear().unwrap();
        assert_eq!(Cache::open(&path).get("keep"), None);
        let _ = std::fs::remove_dir_all(dir);
    }
}
