pub mod esi;
pub mod zkill;

use log::{debug, warn};
use reqwest::Client;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::sync::OnceLock;

use crate::cache::Cache;

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

/// Serialize `value` and store it in the cache under `key` with the given
/// TTL. Caching is best-effort: failures are logged and never propagated.
pub fn cache_set<T: Serialize>(cache: &Cache, key: &str, value: &T, ttl_secs: u64) {
    match serde_json::to_value(value) {
        Ok(json) => {
            cache.set(key, json, ttl_secs);
            debug!("Cached {} for {}s", key, ttl_secs);
        }
        Err(e) => warn!("Failed to serialize cache entry {}: {}", key, e),
    }
}

/// Read a cache entry and deserialize it into `T`. Returns None on a cache
/// miss or when the stored JSON no longer matches `T`.
pub fn cache_get_json<T: DeserializeOwned>(cache: &Cache, key: &str) -> Option<T> {
    serde_json::from_value(cache.get(key)?).ok()
}
