use log::{debug, error, warn};
use reqwest::Client;
use tauri::AppHandle;
use tauri_plugin_cache::CacheExt;

use crate::models::{ActivityHeatmap, ShipStats, SystemStats, ZkillStats};

const DEFAULT_TTL_SECS: u64 = 3600;
const EMPTY_TTL_SECS: u64 = 300;

pub struct FetchResult {
    pub stats: ZkillStats,
    pub from_cache: bool,
}

pub fn try_get_cached(app: &AppHandle, character_id: i64) -> Option<ZkillStats> {
    let cache_key = format!("zkill:{}", character_id);
    let cache = app.cache();

    if let Ok(Some(cached_value)) = cache.get(&cache_key) {
        if let Ok(cached) = serde_json::from_value::<ZkillStats>(cached_value) {
            return Some(cached);
        }
    }
    None
}

pub async fn fetch_stats(
    app: &AppHandle,
    client: &Client,
    character_id: i64,
) -> Result<FetchResult, String> {
    let cache_key = format!("zkill:{}", character_id);
    let cache = app.cache();

    if let Ok(Some(cached_value)) = cache.get(&cache_key) {
        if let Ok(cached) = serde_json::from_value::<ZkillStats>(cached_value) {
            debug!("Cache HIT for zKill {}", character_id);
            return Ok(FetchResult {
                stats: cached,
                from_cache: true,
            });
        }
    }

    let url = format!(
        "https://zkillboard.com/api/stats/characterID/{}/",
        character_id
    );
    debug!("Fetching zKill stats for character {}", character_id);

    let response = client.get(&url).send().await.map_err(|e| {
        error!("zKill request failed for {}: {}", character_id, e);
        format!("Failed to fetch zKill stats: {}", e)
    })?;

    if !response.status().is_success() {
        warn!(
            "zKill returned non-success status {} for character {}",
            response.status(),
            character_id
        );
        return Ok(FetchResult {
            stats: ZkillStats::default(),
            from_cache: false,
        });
    }

    let ttl_secs = parse_max_age_secs(
        response
            .headers()
            .get("cache-control")
            .and_then(|h| h.to_str().ok()),
    )
    .unwrap_or(DEFAULT_TTL_SECS);

    let text = response.text().await.map_err(|e| {
        error!("Failed to read zKill response for {}: {}", character_id, e);
        format!("Failed to read zKill response: {}", e)
    })?;

    if text.is_empty() || text == "[]" {
        debug!("No zKill data for character {}", character_id);
        let stats = ZkillStats::default();

        let options = Some(tauri_plugin_cache::SetItemOptions {
            ttl: Some(EMPTY_TTL_SECS),
            compress: None,
            compression_method: None,
        });
        let _ = cache.set(cache_key, serde_json::to_value(&stats).unwrap(), options);

        return Ok(FetchResult {
            stats,
            from_cache: false,
        });
    }

    let json: serde_json::Value = serde_json::from_str(&text).map_err(|e| {
        error!("Failed to parse zKill JSON for {}: {}", character_id, e);
        format!("Failed to parse zKill JSON: {}", e)
    })?;

    let stats = parse_zkill_response(&json);

    let options = Some(tauri_plugin_cache::SetItemOptions {
        ttl: Some(ttl_secs),
        compress: Some(true),
        compression_method: None,
    });

    if let Err(e) = cache.set(
        cache_key.clone(),
        serde_json::to_value(&stats).unwrap(),
        options,
    ) {
        warn!("Failed to cache zKill {}: {}", character_id, e);
    } else {
        debug!("Cached zKill {} for {}s", character_id, ttl_secs);
    }

    Ok(FetchResult {
        stats,
        from_cache: false,
    })
}

fn parse_max_age_secs(header: Option<&str>) -> Option<u64> {
    let header = header?;
    for part in header.split(',') {
        let part = part.trim();
        if part.starts_with("max-age=") {
            return part.strip_prefix("max-age=")?.parse().ok();
        }
    }
    None
}

fn parse_zkill_response(json: &serde_json::Value) -> ZkillStats {
    let ships_destroyed = json
        .get("shipsDestroyed")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let ships_lost = json.get("shipsLost").and_then(|v| v.as_i64()).unwrap_or(0);
    let isk_destroyed = json
        .get("iskDestroyed")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let isk_lost = json.get("iskLost").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let solo_kills = json.get("soloKills").and_then(|v| v.as_i64()).unwrap_or(0);
    let solo_losses = json.get("soloLosses").and_then(|v| v.as_i64()).unwrap_or(0);
    let danger_ratio = json
        .get("dangerRatio")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let gang_ratio = json
        .get("gangRatio")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let points_destroyed = json
        .get("pointsDestroyed")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let active_pvp_kills = json
        .get("activepvp")
        .and_then(|v| v.get("kills"))
        .and_then(|v| v.get("count"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    let top_ships = parse_top_ships(json);
    let top_systems = parse_top_systems(json);
    let activity = parse_activity(json);

    ZkillStats {
        ships_destroyed,
        ships_lost,
        isk_destroyed,
        isk_lost,
        solo_kills,
        solo_losses,
        danger_ratio,
        gang_ratio,
        points_destroyed,
        active_pvp_kills,
        top_ships,
        activity,
        top_systems,
    }
}

fn parse_top_ships(json: &serde_json::Value) -> Vec<ShipStats> {
    let mut top_ships = Vec::new();

    if let Some(lists) = json.get("topLists").and_then(|v| v.as_array()) {
        for list in lists {
            if list.get("type").and_then(|v| v.as_str()) == Some("shipType") {
                if let Some(values) = list.get("values").and_then(|v| v.as_array()) {
                    for (i, ship) in values.iter().enumerate() {
                        if i >= 5 {
                            break;
                        }
                        let ship_type_id =
                            ship.get("shipTypeID").and_then(|v| v.as_i64()).unwrap_or(0);
                        let ship_name = ship
                            .get("shipName")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown")
                            .to_string();
                        let kills = ship.get("kills").and_then(|v| v.as_i64()).unwrap_or(0);
                        let losses = ship.get("losses").and_then(|v| v.as_i64()).unwrap_or(0);

                        if ship_type_id > 0 {
                            top_ships.push(ShipStats {
                                ship_type_id,
                                ship_name,
                                kills,
                                losses,
                            });
                        }
                    }
                }
            }
        }
    }

    top_ships
}

fn parse_top_systems(json: &serde_json::Value) -> Vec<SystemStats> {
    let mut top_systems = Vec::new();

    if let Some(lists) = json.get("topLists").and_then(|v| v.as_array()) {
        for list in lists {
            if list.get("type").and_then(|v| v.as_str()) == Some("solarSystem") {
                if let Some(values) = list.get("values").and_then(|v| v.as_array()) {
                    for (i, sys) in values.iter().enumerate() {
                        if i >= 5 {
                            break;
                        }
                        let system_id = sys
                            .get("solarSystemID")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0);
                        let system_name = sys
                            .get("solarSystemName")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown")
                            .to_string();
                        let kills = sys.get("kills").and_then(|v| v.as_i64()).unwrap_or(0);

                        if system_id > 0 {
                            top_systems.push(SystemStats {
                                system_id,
                                system_name,
                                kills,
                            });
                        }
                    }
                }
            }
        }
    }

    top_systems
}

fn parse_activity(json: &serde_json::Value) -> Option<ActivityHeatmap> {
    let activity = json.get("activity")?;
    let max = activity.get("max").and_then(|v| v.as_i64()).unwrap_or(1);

    let mut data: Vec<Vec<i64>> = vec![vec![0; 24]; 7];

    for day in 0..7 {
        if let Some(day_data) = activity.get(day.to_string()).and_then(|v| v.as_object()) {
            for (hour_str, count) in day_data {
                if let Ok(hour) = hour_str.parse::<usize>() {
                    if hour < 24 {
                        data[day][hour] = count.as_i64().unwrap_or(0);
                    }
                }
            }
        }
    }

    Some(ActivityHeatmap { max, data })
}
