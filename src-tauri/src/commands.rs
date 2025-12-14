use log::{debug, error, info, warn};
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tauri_plugin_cache::CacheExt;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};

use crate::api::{create_client, esi, zkill};
use crate::intel::calculate_threat_level;
use crate::models::{CharacterInfo, PilotIntel};

const DISPATCH_INTERVAL_MS: u64 = 100;

#[derive(Clone, Serialize)]
pub struct LookupProgress {
    pub current: usize,
    pub total: usize,
    pub cache_hits: usize,
}

#[derive(Clone, Serialize)]
pub struct PilotResult {
    pub pilot: PilotIntel,
    pub index: usize,
}

#[tauri::command]
pub async fn lookup_pilots(app: AppHandle, names_text: String) -> Result<Vec<PilotIntel>, String> {
    let client = create_client()?;

    let names: Vec<String> = names_text
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if names.is_empty() {
        info!("No pilot names provided");
        return Ok(Vec::new());
    }

    let total = names.len();
    info!("Looking up {} pilots", total);
    debug!("Pilot names: {:?}", names);

    let id_map = esi::resolve_character_ids(&client, &names).await?;
    info!("Resolved {} character IDs", id_map.len());

    let mut results: Vec<PilotIntel> = Vec::with_capacity(total);
    let mut uncached: Vec<(usize, String, Option<i64>)> = Vec::new();
    let mut cache_hits: usize = 0;

    for (i, name) in names.into_iter().enumerate() {
        let character_id = id_map.get(&name.to_lowercase()).copied();

        if let Some(pilot) = try_from_cache(&app, character_id) {
            cache_hits += 1;

            let _ = app.emit(
                "pilot-result",
                PilotResult {
                    pilot: pilot.clone(),
                    index: i,
                },
            );

            let _ = app.emit(
                "lookup-progress",
                LookupProgress {
                    current: cache_hits,
                    total,
                    cache_hits,
                },
            );

            results.push(pilot);
        } else {
            uncached.push((i, name, character_id));
        }
    }

    let uncached_count = uncached.len();

    debug!(
        "{} pilots to fetch from API ({} cache hits)",
        uncached_count, cache_hits
    );

    if uncached_count > 0 {
        let (tx, mut rx) = mpsc::channel::<(usize, PilotIntel)>(uncached_count);

        let dispatch_app = app.clone();
        let dispatch_client = client.clone();

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_millis(DISPATCH_INTERVAL_MS));

            for (i, name, character_id) in uncached {
                ticker.tick().await;

                let tx = tx.clone();
                let app_clone = dispatch_app.clone();
                let client_clone = dispatch_client.clone();

                tokio::spawn(async move {
                    let (pilot, _) =
                        fetch_pilot_intel(&app_clone, &client_clone, name, character_id).await;
                    let _ = tx.send((i, pilot)).await;
                });
            }
        });

        let mut received = 0;
        while received < uncached_count {
            if let Some((index, pilot)) = rx.recv().await {
                received += 1;

                let _ = app.emit(
                    "pilot-result",
                    PilotResult {
                        pilot: pilot.clone(),
                        index,
                    },
                );

                let _ = app.emit(
                    "lookup-progress",
                    LookupProgress {
                        current: cache_hits + received,
                        total,
                        cache_hits,
                    },
                );

                results.push(pilot);
            } else {
                break;
            }
        }
    }

    results.sort_by(|a, b| {
        let threat_order = |t: &str| match t {
            "EXTREME" => 0,
            "HIGH" => 1,
            "MODERATE" => 2,
            "LOW" => 3,
            "MINIMAL" => 4,
            _ => 5,
        };
        threat_order(&a.threat_level).cmp(&threat_order(&b.threat_level))
    });

    info!(
        "Lookup complete, returning {} results ({} cache hits)",
        results.len(),
        cache_hits
    );
    Ok(results)
}

fn try_from_cache(app: &AppHandle, character_id: Option<i64>) -> Option<PilotIntel> {
    let id = character_id?;

    let character = esi::try_get_cached_character(app, id)?;
    let zkill_result = zkill::try_get_cached(app, id)?;

    let threat_level = calculate_threat_level(&Some(zkill_result.clone()));

    Some(PilotIntel {
        character,
        zkill: Some(zkill_result),
        threat_level,
        error: None,
    })
}

async fn fetch_pilot_intel(
    app: &AppHandle,
    client: &reqwest::Client,
    name: String,
    character_id: Option<i64>,
) -> (PilotIntel, bool) {
    match character_id {
        Some(id) => match esi::fetch_character_info(app, client, id).await {
            Ok(character) => {
                debug!("Fetched ESI info for {} (ID: {})", character.name, id);

                let (zkill, from_cache) = match zkill::fetch_stats(app, client, id).await {
                    Ok(result) => {
                        debug!(
                            "Fetched zKill stats for {} - {} kills, {} losses (cached: {})",
                            character.name,
                            result.stats.ships_destroyed,
                            result.stats.ships_lost,
                            result.from_cache
                        );
                        (Some(result.stats), result.from_cache)
                    }
                    Err(e) => {
                        warn!(
                            "Failed to fetch zKill stats for {} (ID: {}): {}",
                            character.name, id, e
                        );
                        (None, false)
                    }
                };

                let threat_level = calculate_threat_level(&zkill);
                (
                    PilotIntel {
                        character,
                        zkill,
                        threat_level,
                        error: None,
                    },
                    from_cache,
                )
            }
            Err(e) => {
                error!("Failed to fetch ESI info for {} (ID: {}): {}", name, id, e);
                (
                    PilotIntel {
                        character: CharacterInfo {
                            id,
                            name,
                            corporation_id: None,
                            corporation_name: None,
                            corporation_ticker: None,
                            alliance_id: None,
                            alliance_name: None,
                            alliance_ticker: None,
                        },
                        zkill: None,
                        threat_level: "Unknown".to_string(),
                        error: Some(e),
                    },
                    false,
                )
            }
        },
        None => {
            warn!("Character not found in ESI: {}", name);
            (
                PilotIntel {
                    character: CharacterInfo {
                        id: 0,
                        name,
                        corporation_id: None,
                        corporation_name: None,
                        corporation_ticker: None,
                        alliance_id: None,
                        alliance_name: None,
                        alliance_ticker: None,
                    },
                    zkill: None,
                    threat_level: "Unknown".to_string(),
                    error: Some("Character not found".to_string()),
                },
                false,
            )
        }
    }
}

#[tauri::command]
pub fn clear_cache(app: AppHandle) -> Result<(), String> {
    let cache = app.cache();
    cache
        .clear()
        .map_err(|e| format!("Failed to clear cache: {}", e))?;
    info!("Cache cleared");
    Ok(())
}

#[derive(Clone, Serialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub release_url: String,
    pub release_notes: String,
}

#[tauri::command]
pub async fn check_for_update() -> Result<Option<UpdateInfo>, String> {
    let client = create_client()?;
    let current_version = env!("CARGO_PKG_VERSION");

    info!("Checking for updates (current: v{})", current_version);

    let response = client
        .get("https://api.github.com/repos/eve-telescope/telescope-app/releases/latest")
        .send()
        .await
        .map_err(|e| {
            warn!("Failed to check for updates: {}", e);
            format!("Failed to check for updates: {}", e)
        })?;

    if !response.status().is_success() {
        warn!("Update check failed: HTTP {}", response.status());
        return Ok(None);
    }

    let release: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse release: {}", e))?;

    let latest_version = release["tag_name"]
        .as_str()
        .unwrap_or("")
        .trim_start_matches('v');

    if is_newer_version(latest_version, current_version) {
        info!(
            "Update available: v{} → v{}",
            current_version, latest_version
        );
        Ok(Some(UpdateInfo {
            current_version: current_version.to_string(),
            latest_version: latest_version.to_string(),
            release_url: release["html_url"].as_str().unwrap_or("").to_string(),
            release_notes: release["body"].as_str().unwrap_or("").to_string(),
        }))
    } else {
        info!("App is up to date (v{})", current_version);
        Ok(None)
    }
}

fn is_newer_version(latest: &str, current: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> { v.split('.').filter_map(|s| s.parse().ok()).collect() };

    let latest_parts = parse(latest);
    let current_parts = parse(current);

    for i in 0..latest_parts.len().max(current_parts.len()) {
        let l = latest_parts.get(i).copied().unwrap_or(0);
        let c = current_parts.get(i).copied().unwrap_or(0);
        if l > c {
            return true;
        }
        if l < c {
            return false;
        }
    }
    false
}
