use log::{debug, error, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::AppHandle;

use super::{cache_get_json, cache_set};
use crate::models::CharacterInfo;

const DEFAULT_TTL_SECS: u64 = 3600;
// Corp/alliance renames and ticker changes shouldn't stay stale for a day;
// an hour is plenty — the cache's main job is deduping affiliation lookups
// within and between scans, not long-term storage.
const AFFILIATION_TTL_SECS: u64 = 3600;

#[derive(Debug, Deserialize)]
struct EsiCharacter {
    name: String,
    corporation_id: i64,
    alliance_id: Option<i64>,
}

/// Corporations and alliances share the same name/ticker shape in ESI.
#[derive(Debug, Serialize, Deserialize)]
struct EsiAffiliation {
    name: String,
    ticker: String,
}

#[derive(Debug, Deserialize)]
struct EsiIdResult {
    characters: Option<Vec<EsiIdEntry>>,
}

#[derive(Debug, Deserialize)]
struct EsiIdEntry {
    id: i64,
    name: String,
}

pub async fn resolve_character_ids(
    client: &Client,
    names: &[String],
) -> Result<HashMap<String, i64>, String> {
    let url = "https://esi.evetech.net/latest/universe/ids/?datasource=tranquility";
    debug!("Resolving {} character names via ESI", names.len());

    let response = client.post(url).json(&names).send().await.map_err(|e| {
        error!("ESI request failed: {}", e);
        format!("Failed to resolve character IDs: {}", e)
    })?;

    if !response.status().is_success() {
        error!("ESI returned error status: {}", response.status());
        return Err(format!("ESI returned error: {}", response.status()));
    }

    let result: EsiIdResult = response.json().await.map_err(|e| {
        error!("Failed to parse ESI response: {}", e);
        format!("Failed to parse ESI response: {}", e)
    })?;

    let mut map = HashMap::new();
    if let Some(characters) = result.characters {
        for entry in characters {
            map.insert(entry.name.to_lowercase(), entry.id);
        }
    }

    let unresolved: Vec<_> = names
        .iter()
        .filter(|n| !map.contains_key(&n.to_lowercase()))
        .collect();

    if !unresolved.is_empty() {
        warn!(
            "Could not resolve {} characters: {:?}",
            unresolved.len(),
            unresolved
        );
    }

    Ok(map)
}

pub fn try_get_cached_character(app: &AppHandle, character_id: i64) -> Option<CharacterInfo> {
    cache_get_json(app, &format!("char:{}", character_id))
}

pub async fn fetch_character_info(
    app: &AppHandle,
    client: &Client,
    character_id: i64,
) -> Result<CharacterInfo, String> {
    if let Some(cached) = try_get_cached_character(app, character_id) {
        debug!("Cache HIT for character {}", character_id);
        return Ok(cached);
    }

    let char_url = format!(
        "https://esi.evetech.net/latest/characters/{}/?datasource=tranquility",
        character_id
    );

    let char_response = client
        .get(&char_url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch character: {}", e))?;

    if !char_response.status().is_success() {
        return Err(format!("Character not found: {}", character_id));
    }

    let ttl_secs = parse_expires_to_secs(
        char_response
            .headers()
            .get("expires")
            .and_then(|h| h.to_str().ok()),
    )
    .unwrap_or(DEFAULT_TTL_SECS);

    let esi_char: EsiCharacter = char_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse character: {}", e))?;

    // Corp and alliance are independent lookups — fetch them concurrently.
    let corp_fut = fetch_affiliation(
        app,
        client,
        format!("corp:{}", esi_char.corporation_id),
        format!(
            "https://esi.evetech.net/latest/corporations/{}/?datasource=tranquility",
            esi_char.corporation_id
        ),
    );
    let alliance_fut = async {
        match esi_char.alliance_id {
            Some(alliance_id) => {
                fetch_affiliation(
                    app,
                    client,
                    format!("alliance:{}", alliance_id),
                    format!(
                        "https://esi.evetech.net/latest/alliances/{}/?datasource=tranquility",
                        alliance_id
                    ),
                )
                .await
            }
            None => (None, None),
        }
    };
    let ((corp_name, corp_ticker), (alliance_name, alliance_ticker)) =
        tokio::join!(corp_fut, alliance_fut);

    let info = CharacterInfo {
        id: character_id,
        name: esi_char.name,
        corporation_id: Some(esi_char.corporation_id),
        corporation_name: corp_name,
        corporation_ticker: corp_ticker,
        alliance_id: esi_char.alliance_id,
        alliance_name,
        alliance_ticker,
    };

    cache_set(
        app,
        &format!("char:{}", character_id),
        &info,
        ttl_secs,
        false,
    );

    Ok(info)
}

/// Fetch a corp or alliance name/ticker with its own cache entry, so pilots
/// sharing an affiliation don't refetch it. Failures degrade to (None, None).
///
/// Note: concurrent lookups of the same affiliation can still race between
/// the cache check and the cache write, causing a few duplicate fetches
/// (bounded by the 8-way lookup concurrency cap). Single-flighting the
/// request per cache key would be the deeper fix.
async fn fetch_affiliation(
    app: &AppHandle,
    client: &Client,
    cache_key: String,
    url: String,
) -> (Option<String>, Option<String>) {
    if let Some(cached) = cache_get_json::<EsiAffiliation>(app, &cache_key) {
        return (Some(cached.name), Some(cached.ticker));
    }

    let affiliation = match client.get(&url).send().await {
        Ok(resp) => match resp.json::<EsiAffiliation>().await {
            Ok(affiliation) => affiliation,
            Err(e) => {
                warn!("Failed to parse {}: {}", cache_key, e);
                return (None, None);
            }
        },
        Err(e) => {
            warn!("Failed to fetch {}: {}", cache_key, e);
            return (None, None);
        }
    };

    cache_set(app, &cache_key, &affiliation, AFFILIATION_TTL_SECS, false);

    (Some(affiliation.name), Some(affiliation.ticker))
}

fn parse_expires_to_secs(header: Option<&str>) -> Option<u64> {
    use chrono::{DateTime, Utc};

    let header = header?;
    let expires: DateTime<Utc> = DateTime::parse_from_rfc2822(header)
        .ok()?
        .with_timezone(&Utc);

    let now = Utc::now();
    if expires > now {
        Some((expires - now).num_seconds() as u64)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::parse_expires_to_secs;
    use chrono::{Duration, Utc};

    #[test]
    fn future_expires_yields_remaining_seconds() {
        let header = (Utc::now() + Duration::seconds(120)).to_rfc2822();
        let secs = parse_expires_to_secs(Some(&header)).expect("future date should parse");
        // Allow slack for test execution time.
        assert!((115..=120).contains(&secs), "got {}", secs);
    }

    #[test]
    fn past_expires_yields_none() {
        let header = (Utc::now() - Duration::seconds(60)).to_rfc2822();
        assert_eq!(parse_expires_to_secs(Some(&header)), None);
    }

    #[test]
    fn missing_or_malformed_header_yields_none() {
        assert_eq!(parse_expires_to_secs(None), None);
        assert_eq!(parse_expires_to_secs(Some("not a date")), None);
        assert_eq!(parse_expires_to_secs(Some("")), None);
    }
}
