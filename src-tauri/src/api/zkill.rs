use log::{debug, error, warn};
use reqwest::Client;
use tauri::AppHandle;

use super::{cache_get_json, cache_set};
use crate::models::{ActivityHeatmap, ShipStats, SystemStats, ZkillStats};

const DEFAULT_TTL_SECS: u64 = 3600;
const EMPTY_TTL_SECS: u64 = 300;

pub struct FetchResult {
    pub stats: ZkillStats,
    pub from_cache: bool,
}

pub fn try_get_cached(app: &AppHandle, character_id: i64) -> Option<ZkillStats> {
    cache_get_json(app, &format!("zkill:{}", character_id))
}

pub async fn fetch_stats(
    app: &AppHandle,
    client: &Client,
    character_id: i64,
) -> Result<FetchResult, String> {
    let cache_key = format!("zkill:{}", character_id);

    if let Some(cached) = try_get_cached(app, character_id) {
        debug!("Cache HIT for zKill {}", character_id);
        return Ok(FetchResult {
            stats: cached,
            from_cache: true,
        });
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

        cache_set(app, &cache_key, &stats, EMPTY_TTL_SECS, false);

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

    cache_set(app, &cache_key, &stats, ttl_secs, true);

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

    let avg_attackers = json
        .get("avgGangSize")
        .and_then(|v| v.as_f64())
        .unwrap_or(1.0);

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
        avg_attackers,
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
                        let group_id = ship.get("groupID").and_then(|v| v.as_i64()).unwrap_or(0);
                        let group_name = ship
                            .get("groupName")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown")
                            .to_string();
                        let kills = ship.get("kills").and_then(|v| v.as_i64()).unwrap_or(0);
                        let losses = ship.get("losses").and_then(|v| v.as_i64()).unwrap_or(0);

                        if ship_type_id > 0 {
                            top_ships.push(ShipStats {
                                ship_type_id,
                                ship_name,
                                group_id,
                                group_name,
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

    for (day, row) in data.iter_mut().enumerate() {
        if let Some(day_data) = activity.get(day.to_string()).and_then(|v| v.as_object()) {
            for (hour_str, count) in day_data {
                if let Ok(hour) = hour_str.parse::<usize>() {
                    if hour < 24 {
                        row[hour] = count.as_i64().unwrap_or(0);
                    }
                }
            }
        }
    }

    Some(ActivityHeatmap { max, data })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_max_age_extracts_value() {
        assert_eq!(parse_max_age_secs(Some("max-age=3600")), Some(3600));
        assert_eq!(
            parse_max_age_secs(Some("public, max-age=600, immutable")),
            Some(600)
        );
        assert_eq!(parse_max_age_secs(Some("no-cache")), None);
        assert_eq!(parse_max_age_secs(Some("max-age=abc")), None);
        assert_eq!(parse_max_age_secs(None), None);
    }

    #[test]
    fn parse_zkill_response_reads_all_fields() {
        let json = json!({
            "shipsDestroyed": 150,
            "shipsLost": 20,
            "iskDestroyed": 1.5e9,
            "iskLost": 2.0e8,
            "soloKills": 40,
            "soloLosses": 5,
            "dangerRatio": 85.0,
            "gangRatio": 60.0,
            "pointsDestroyed": 5000,
            "activepvp": { "kills": { "count": 30 } },
            "avgGangSize": 3.5
        });
        let stats = parse_zkill_response(&json);
        assert_eq!(stats.ships_destroyed, 150);
        assert_eq!(stats.ships_lost, 20);
        assert_eq!(stats.isk_destroyed, 1.5e9);
        assert_eq!(stats.solo_kills, 40);
        assert_eq!(stats.danger_ratio, 85.0);
        assert_eq!(stats.active_pvp_kills, 30);
        assert_eq!(stats.avg_attackers, 3.5);
        assert!(stats.top_ships.is_empty());
        assert!(stats.activity.is_none());
    }

    #[test]
    fn parse_zkill_response_defaults_on_missing_fields() {
        let stats = parse_zkill_response(&json!({}));
        assert_eq!(stats.ships_destroyed, 0);
        assert_eq!(stats.isk_destroyed, 0.0);
        assert_eq!(stats.avg_attackers, 1.0);
        assert!(stats.top_ships.is_empty());
        assert!(stats.top_systems.is_empty());
        assert!(stats.activity.is_none());
    }

    fn ship(id: i64, name: &str, kills: i64) -> serde_json::Value {
        json!({
            "shipTypeID": id,
            "shipName": name,
            "groupID": 26,
            "groupName": "Cruiser",
            "kills": kills,
            "losses": 1
        })
    }

    #[test]
    fn parse_top_ships_truncates_to_five_and_skips_invalid() {
        let values: Vec<_> = (1..=7).map(|i| ship(i, "Ship", i * 10)).collect();
        let json = json!({
            "topLists": [{ "type": "shipType", "values": values }]
        });
        let ships = parse_top_ships(&json);
        assert_eq!(ships.len(), 5);
        assert_eq!(ships[0].kills, 10);

        // shipTypeID 0 (or missing) rows are dropped.
        let json = json!({
            "topLists": [{ "type": "shipType", "values": [ship(0, "Bad", 5), ship(3, "Ok", 5)] }]
        });
        let ships = parse_top_ships(&json);
        assert_eq!(ships.len(), 1);
        assert_eq!(ships[0].ship_type_id, 3);
    }

    #[test]
    fn parse_top_ships_ignores_other_list_types() {
        let json = json!({
            "topLists": [{ "type": "solarSystem", "values": [ship(1, "Ship", 5)] }]
        });
        assert!(parse_top_ships(&json).is_empty());
    }

    #[test]
    fn parse_top_systems_reads_and_truncates() {
        let values: Vec<_> = (1..=6)
            .map(|i| {
                json!({
                    "solarSystemID": 30000000 + i,
                    "solarSystemName": format!("System {}", i),
                    "kills": i
                })
            })
            .collect();
        let json = json!({
            "topLists": [{ "type": "solarSystem", "values": values }]
        });
        let systems = parse_top_systems(&json);
        assert_eq!(systems.len(), 5);
        assert_eq!(systems[0].system_name, "System 1");
    }

    #[test]
    fn parse_activity_builds_7x24_grid() {
        let json = json!({
            "activity": {
                "max": 12,
                "0": { "5": 3, "23": 7 },
                "6": { "0": 1 },
                "9": { "0": 99 },          // day out of range: ignored
                "1": { "24": 5, "x": 2 }   // hour out of range / non-numeric: ignored
            }
        });
        let heatmap = parse_activity(&json).expect("activity should parse");
        assert_eq!(heatmap.max, 12);
        assert_eq!(heatmap.data.len(), 7);
        assert_eq!(heatmap.data[0][5], 3);
        assert_eq!(heatmap.data[0][23], 7);
        assert_eq!(heatmap.data[6][0], 1);
        assert_eq!(heatmap.data[1].iter().sum::<i64>(), 0);
    }

    #[test]
    fn parse_activity_missing_yields_none() {
        assert!(parse_activity(&json!({})).is_none());
    }
}
