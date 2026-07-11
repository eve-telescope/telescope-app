//! Pilot lookup pipeline: resolve names, serve from cache, fetch the rest
//! with paced concurrency, and stream batched results/progress to the
//! frontend as "pilot-batch" events (see [`PilotBatch`]).

use futures::StreamExt;
use log::{debug, error, info, warn};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use std::collections::VecDeque;

use crate::api::{create_client, esi, zkill};
use crate::domain::lookup::{
    LookupEvent, LookupProgress, LookupTracker, BATCH_INTERVAL_MS, MAX_BATCH_SIZE,
};
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

/// Payload of the "pilot-batch" event: every pilot completed since the
/// previous batch, plus the progress snapshot as of the last one. Replaces
/// the former per-pilot "pilot-result" + "lookup-progress" event pair
/// (2N IPC messages per scan) with at most one message per
/// [`crate::domain::lookup::BATCH_INTERVAL_MS`].
#[derive(Clone, Serialize)]
pub struct PilotBatch {
    pub pilots: Vec<PilotResult>,
    pub progress: LookupProgress,
}

fn emit_batch(app: &AppHandle, pilots: Vec<PilotResult>, progress: LookupProgress) {
    let _ = app.emit("pilot-batch", PilotBatch { pilots, progress });
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

    // One delivery mode: every result — cached or fetched — enters this
    // queue and leaves as a "pilot-batch" of at most MAX_BATCH_SIZE per
    // BATCH_INTERVAL_MS tick. A hot cache doesn't teleport the list in as
    // one blob; it just drains the stream at full cadence, so the UI
    // animates identically regardless of where results came from.
    let mut queue: VecDeque<PilotResult> = VecDeque::new();

    for (i, name) in names.into_iter().enumerate() {
        let character_id = id_map.get(&name.to_lowercase()).copied();

        if let Some(pilot) = try_from_cache(&app, character_id) {
            tracker.apply(LookupEvent::CacheHit);
            queue.push_back(PilotResult {
                pilot: pilot.clone(),
                index: i,
            });
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

    let mut stream_done = false;
    let mut ticker = tokio::time::interval(std::time::Duration::from_millis(BATCH_INTERVAL_MS));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    while !(stream_done && queue.is_empty()) {
        tokio::select! {
            _ = ticker.tick() => {
                if !queue.is_empty() {
                    let take = queue.len().min(MAX_BATCH_SIZE);
                    let batch: Vec<PilotResult> = queue.drain(..take).collect();
                    emit_batch(&app, batch, tracker.progress());
                }
            }
            next = lookups.next(), if !stream_done => {
                match next {
                    Some((index, pilot)) => {
                        tracker.apply(LookupEvent::Fetched);
                        queue.push_back(PilotResult {
                            pilot: pilot.clone(),
                            index,
                        });
                        results.push(pilot);
                    }
                    None => stream_done = true,
                }
            }
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

    // Borrow for scoring, then move into the struct — no clone of the
    // stats (the activity heatmap makes that clone expensive) on the
    // warm-cache path.
    let zkill_opt = Some(zkill_result);
    let threat_level = calculate_threat_level(&zkill_opt);
    let flags = detect_pilot_flags(&zkill_opt);

    Some(PilotIntel {
        character,
        zkill: zkill_opt,
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
