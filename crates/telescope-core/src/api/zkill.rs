use crate::cache::Cache;
use log::{debug, error, warn};
use reqwest::Client;

use super::{cache_get_json, cache_set};
use crate::models::{ActivityHeatmap, GroupLosses, ShipStats, SystemStats, ZkillStats};

const DEFAULT_TTL_SECS: u64 = 3600;
const EMPTY_TTL_SECS: u64 = 300;

pub struct FetchResult {
    pub stats: ZkillStats,
    pub from_cache: bool,
}

/// Versioned so entries parsed before zKillboard moved ship stats into
/// `topShips` are not reused.
fn cache_key(character_id: i64) -> String {
    format!("zkill:v4:{character_id}")
}

pub fn try_get_cached(cache: &Cache, character_id: i64) -> Option<ZkillStats> {
    cache_get_json(cache, &cache_key(character_id))
}

pub async fn fetch_stats(
    cache: &Cache,
    client: &Client,
    character_id: i64,
) -> Result<FetchResult, String> {
    let cache_key = cache_key(character_id);

    if let Some(cached) = try_get_cached(cache, character_id) {
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

        cache_set(cache, &cache_key, &stats, EMPTY_TTL_SECS);

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

    cache_set(cache, &cache_key, &stats, ttl_secs);

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
    let lost_groups = parse_lost_groups(json);
    let recent_kills = parse_recent_kills(json, chrono::Utc::now().date_naive());
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
        lost_groups,
        recent_kills,
    }
}

/// Kills in `today`'s calendar month and the two before it, from the
/// `months` history keyed `YYYYMM`.
fn parse_recent_kills(json: &serde_json::Value, today: chrono::NaiveDate) -> i64 {
    use chrono::Datelike;
    let month_index = |year: i32, month: u32| year * 12 + month as i32 - 1;
    let current = month_index(today.year(), today.month());
    json.get("months")
        .and_then(|v| v.as_object())
        .into_iter()
        .flat_map(|months| months.values())
        .filter_map(|month| {
            let year = month.get("year")?.as_i64()? as i32;
            let number = month.get("month")?.as_i64()? as u32;
            let age = current - month_index(year, number);
            (0..3).contains(&age).then(|| {
                month
                    .get("shipsDestroyed")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0)
            })
        })
        .sum()
}

/// `groups` holds all-time stats per ship group; `shipsLost` there counts
/// the pilot's own losses in that group, so it tells what they have flown.
fn parse_lost_groups(json: &serde_json::Value) -> Vec<GroupLosses> {
    let mut groups: Vec<GroupLosses> = json
        .get("groups")
        .and_then(|v| v.as_object())
        .into_iter()
        .flat_map(|groups| groups.values())
        .filter_map(|group| {
            let group_id = group.get("groupID")?.as_i64()?;
            let losses = group.get("shipsLost")?.as_i64()?;
            (losses > 0).then_some(GroupLosses { group_id, losses })
        })
        .collect();
    groups.sort_by_key(|g| g.group_id);
    groups
}

/// Ships the pilot flew, most appearances first. zKillboard now reports
/// them in `topShips` without names (filled in from the SDE later); the
/// older `topLists` shipType list is read when `topShips` is absent.
fn parse_top_ships(json: &serde_json::Value) -> Vec<ShipStats> {
    let rows: Vec<&serde_json::Value> = match json.get("topShips").and_then(|v| v.as_array()) {
        Some(rows) => rows.iter().collect(),
        None => json
            .get("topLists")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
            .filter(|list| list.get("type").and_then(|v| v.as_str()) == Some("shipType"))
            .filter_map(|list| list.get("values").and_then(|v| v.as_array()))
            .flatten()
            .collect(),
    };

    let int =
        |row: &serde_json::Value, key: &str| row.get(key).and_then(|v| v.as_i64()).unwrap_or(0);
    let text = |row: &serde_json::Value, key: &str| {
        row.get(key)
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string()
    };

    let mut ships: Vec<ShipStats> = rows
        .into_iter()
        .map(|row| ShipStats {
            ship_type_id: int(row, "shipTypeID"),
            ship_name: text(row, "shipName"),
            group_id: int(row, "groupID"),
            group_name: text(row, "groupName"),
            kills: int(row, "kills"),
            losses: int(row, "losses"),
        })
        .filter(|ship| ship.ship_type_id > 0)
        .collect();
    ships.sort_by_key(|ship| std::cmp::Reverse(ship.kills + ship.losses));
    ships.truncate(10);
    ships
}

fn parse_top_systems(json: &serde_json::Value) -> Vec<SystemStats> {
    let mut top_systems = Vec::new();

    if let Some(lists) = json.get("topLists").and_then(|v| v.as_array()) {
        for list in lists {
            if list.get("type").and_then(|v| v.as_str()) == Some("solarSystem")
                && let Some(values) = list.get("values").and_then(|v| v.as_array())
            {
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

    top_systems
}

fn parse_activity(json: &serde_json::Value) -> Option<ActivityHeatmap> {
    let activity = json.get("activity")?;
    let max = activity.get("max").and_then(|v| v.as_i64()).unwrap_or(1);

    let mut data: Vec<Vec<i64>> = vec![vec![0; 24]; 7];

    for (day, row) in data.iter_mut().enumerate() {
        if let Some(day_data) = activity.get(day.to_string()).and_then(|v| v.as_object()) {
            for (hour_str, count) in day_data {
                if let Ok(hour) = hour_str.parse::<usize>()
                    && hour < 24
                {
                    row[hour] = count.as_i64().unwrap_or(0);
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
    fn parse_top_ships_reads_top_ships_by_appearances() {
        let json = json!({
            "topShips": [
                { "shipTypeID": 608, "groupID": 25, "kills": 0, "losses": 1 },
                { "shipTypeID": 11957, "groupID": 833, "kills": 4, "losses": 2 },
                { "shipTypeID": 0, "groupID": 25, "kills": 9, "losses": 0 }
            ]
        });
        let ships = parse_top_ships(&json);
        assert_eq!(ships.len(), 2);
        assert_eq!(ships[0].ship_type_id, 11957);
        assert_eq!(ships[0].group_id, 833);
        assert_eq!(ships[0].ship_name, "");
    }

    #[test]
    fn parse_top_ships_truncates_to_ten() {
        let rows: Vec<_> = (1..=12).map(|i| ship(i, "Ship", i * 10)).collect();
        let ships = parse_top_ships(&json!({ "topShips": rows }));
        assert_eq!(ships.len(), 10);
        assert_eq!(ships[0].kills, 120);
    }

    #[test]
    fn parse_recent_kills_sums_the_last_three_months() {
        let json = json!({
            "months": {
                "202610": { "year": 2026, "month": 10, "shipsDestroyed": 5 },
                "202609": { "year": 2026, "month": 9, "shipsDestroyed": 7 },
                "202608": { "year": 2026, "month": 8, "shipsDestroyed": 11 },
                "202607": { "year": 2026, "month": 7, "shipsDestroyed": 100 },
                "202512": { "year": 2025, "month": 12, "shipsLost": 3 },
                "202612": { "year": 2026, "month": 12, "shipsDestroyed": 2 }
            }
        });
        let today = chrono::NaiveDate::from_ymd_opt(2026, 10, 4).unwrap();
        assert_eq!(parse_recent_kills(&json, today), 23);

        // December counts across the year boundary; October is too old.
        let february = chrono::NaiveDate::from_ymd_opt(2027, 2, 1).unwrap();
        assert_eq!(parse_recent_kills(&json, february), 2);
    }

    #[test]
    fn parse_lost_groups_keeps_groups_with_losses() {
        let json = json!({
            "groups": {
                "898": { "groupID": 898, "shipsLost": 2, "shipsDestroyed": 0 },
                "30": { "groupID": 30, "shipsLost": 0, "shipsDestroyed": 5 },
                "26": { "groupID": 26, "shipsLost": 7 }
            }
        });
        assert_eq!(
            parse_lost_groups(&json),
            vec![
                GroupLosses {
                    group_id: 26,
                    losses: 7
                },
                GroupLosses {
                    group_id: 898,
                    losses: 2
                },
            ]
        );
    }

    #[test]
    fn parse_top_ships_falls_back_to_top_lists() {
        let json = json!({
            "topLists": [
                { "type": "solarSystem", "values": [ship(1, "Ignored", 5)] },
                { "type": "shipType", "values": [ship(3, "Rifter", 5)] }
            ]
        });
        let ships = parse_top_ships(&json);
        assert_eq!(ships.len(), 1);
        assert_eq!(ships[0].ship_name, "Rifter");
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
