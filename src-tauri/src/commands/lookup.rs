//! Pilot lookup pipeline: resolve names, serve from cache, fetch the rest
//! with paced concurrency, and stream results/progress to the frontend.

use futures::StreamExt;
use log::{debug, error, info, warn};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::api::{create_client, esi, zkill};
use crate::domain::lookup::{LookupEvent, LookupTracker};
use crate::domain::threat::{calculate_threat_level, detect_pilot_flags};
use crate::models::{CharacterInfo, PilotFlags, PilotIntel};

/// Cap on simultaneous per-pilot lookups so large locals don't burst
/// hundreds of concurrent ESI/zKill requests into rate limits.
const MAX_CONCURRENT_LOOKUPS: usize = 8;

/// Minimum spacing between lookup launches. Pacing launches to one every
/// 100ms approximates ≤10 launches/sec for zKillboard rate-limit safety,
/// while `buffer_unordered` above caps the number in flight at 8.
const DISPATCH_INTERVAL_MS: u64 = 100;

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
    let mut tracker = LookupTracker::new(total);

    for (i, name) in names.into_iter().enumerate() {
        let character_id = id_map.get(&name.to_lowercase()).copied();

        if let Some(pilot) = try_from_cache(&app, character_id) {
            let progress = tracker.apply(LookupEvent::CacheHit);

            let _ = app.emit(
                "pilot-result",
                PilotResult {
                    pilot: pilot.clone(),
                    index: i,
                },
            );

            let _ = app.emit("lookup-progress", progress);

            results.push(pilot);
        } else {
            uncached.push((i, name, character_id));
        }
    }

    debug!(
        "{} pilots to fetch from API ({} cache hits)",
        uncached.len(),
        tracker.cache_hits()
    );

    if !tracker.is_complete() {
        let started = tokio::time::Instant::now();
        let mut lookups = futures::stream::iter(uncached.into_iter().enumerate().map(
            |(launch_index, (i, name, character_id))| {
                let app = app.clone();
                let client = client.clone();
                async move {
                    // Pace launches: the n-th lookup may not start before
                    // n * DISPATCH_INTERVAL_MS after the stream began.
                    tokio::time::sleep_until(
                        started
                            + std::time::Duration::from_millis(
                                launch_index as u64 * DISPATCH_INTERVAL_MS,
                            ),
                    )
                    .await;
                    let (pilot, _) = fetch_pilot_intel(&app, &client, name, character_id).await;
                    (i, pilot)
                }
            },
        ))
        .buffer_unordered(MAX_CONCURRENT_LOOKUPS);

        while let Some((index, pilot)) = lookups.next().await {
            let progress = tracker.apply(LookupEvent::Fetched);

            let _ = app.emit(
                "pilot-result",
                PilotResult {
                    pilot: pilot.clone(),
                    index,
                },
            );

            let _ = app.emit("lookup-progress", progress);

            results.push(pilot);
        }
    }

    results.sort_by_key(|pilot| crate::domain::threat::threat_rank(&pilot.threat_level));

    info!(
        "Lookup complete, returning {} results ({} cache hits)",
        results.len(),
        tracker.cache_hits()
    );
    Ok(results)
}

fn try_from_cache(app: &AppHandle, character_id: Option<i64>) -> Option<PilotIntel> {
    let id = character_id?;

    let character = esi::try_get_cached_character(app, id)?;
    let zkill_result = zkill::try_get_cached(app, id)?;

    let zkill_opt = Some(zkill_result.clone());
    let threat_level = calculate_threat_level(&zkill_opt);
    let flags = detect_pilot_flags(&zkill_opt);

    Some(PilotIntel {
        character,
        zkill: Some(zkill_result),
        threat_level,
        flags,
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
                let flags = detect_pilot_flags(&zkill);
                (
                    PilotIntel {
                        character,
                        zkill,
                        threat_level,
                        flags,
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
                        flags: PilotFlags::default(),
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
                    flags: PilotFlags::default(),
                    error: Some("Character not found".to_string()),
                },
                false,
            )
        }
    }
}
