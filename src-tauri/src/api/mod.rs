pub mod esi;
pub mod zkill;

use log::{debug, warn};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::OnceLock;
use tauri::AppHandle;
use tauri_plugin_cache::CacheExt;

const VERSION: &str = env!("CARGO_PKG_VERSION");

static SHARED_CLIENT: OnceLock<Client> = OnceLock::new();

/// Shared client for public APIs (ESI, zKillboard, GitHub, SDE host).
/// Reusing one client keeps the connection pool and TLS session alive
/// across commands instead of rebuilding them per request.
pub fn create_client() -> Result<Client, String> {
    if let Some(client) = SHARED_CLIENT.get() {
        return Ok(client.clone());
    }

    let user_agent = format!(
        "Telescope/{} (eve-telescope.com; github.com/eve-telescope/telescope-app)",
        VERSION
    );

    let client = Client::builder()
        .user_agent(user_agent)
        .build()
        .map_err(|e| e.to_string())?;

    Ok(SHARED_CLIENT.get_or_init(|| client).clone())
}

/// Serialize `value` and store it in the app cache under `key` with the given
/// TTL. Caching is best-effort: failures are logged and never propagated.
pub fn cache_set<T: Serialize>(
    app: &AppHandle,
    key: &str,
    value: &T,
    ttl_secs: u64,
    compress: bool,
) {
    let json = match serde_json::to_value(value) {
        Ok(json) => json,
        Err(e) => {
            warn!("Failed to serialize cache entry {}: {}", key, e);
            return;
        }
    };

    let options = Some(tauri_plugin_cache::SetItemOptions {
        ttl: Some(ttl_secs),
        compress: compress.then_some(true),
        compression_method: None,
    });

    if let Err(e) = app.cache().set(key.to_string(), json, options) {
        warn!("Failed to cache {}: {}", key, e);
    } else {
        debug!("Cached {} for {}s", key, ttl_secs);
    }
}

/// Read a cache entry and deserialize it into `T`. Returns None on a cache
/// miss, cache error, or when the stored JSON no longer matches `T`.
pub fn cache_get_json<T: DeserializeOwned>(app: &AppHandle, key: &str) -> Option<T> {
    let cached = app.cache().get(key).ok().flatten()?;
    serde_json::from_value(cached).ok()
}
