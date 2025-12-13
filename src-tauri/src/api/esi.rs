use log::{debug, warn, error};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use tauri::AppHandle;
use tauri_plugin_cache::CacheExt;

use crate::models::CharacterInfo;

const DEFAULT_TTL_SECS: u64 = 3600;

#[derive(Debug, Deserialize)]
struct EsiCharacter {
    name: String,
    corporation_id: i64,
    alliance_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct EsiCorporation {
    name: String,
    ticker: String,
}

#[derive(Debug, Deserialize)]
struct EsiAlliance {
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

    let response = client
        .post(url)
        .json(&names)
        .send()
        .await
        .map_err(|e| {
            error!("ESI request failed: {}", e);
            format!("Failed to resolve character IDs: {}", e)
        })?;

    if !response.status().is_success() {
        error!("ESI returned error status: {}", response.status());
        return Err(format!("ESI returned error: {}", response.status()));
    }

    let result: EsiIdResult = response
        .json()
        .await
        .map_err(|e| {
            error!("Failed to parse ESI response: {}", e);
            format!("Failed to parse ESI response: {}", e)
        })?;

    let mut map = HashMap::new();
    if let Some(characters) = result.characters {
        for entry in characters {
            map.insert(entry.name.to_lowercase(), entry.id);
        }
    }

    let unresolved: Vec<_> = names.iter()
        .filter(|n| !map.contains_key(&n.to_lowercase()))
        .collect();
    
    if !unresolved.is_empty() {
        warn!("Could not resolve {} characters: {:?}", unresolved.len(), unresolved);
    }

    Ok(map)
}

pub async fn fetch_character_info(
    app: &AppHandle,
    client: &Client,
    character_id: i64,
) -> Result<CharacterInfo, String> {
    let cache_key = format!("char:{}", character_id);
    let cache = app.cache();
    
    if let Ok(Some(cached_value)) = cache.get(&cache_key) {
        if let Ok(cached) = serde_json::from_value::<CharacterInfo>(cached_value) {
            debug!("Cache HIT for character {}", character_id);
            return Ok(cached);
        }
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
        char_response.headers().get("expires").and_then(|h| h.to_str().ok())
    ).unwrap_or(DEFAULT_TTL_SECS);

    let esi_char: EsiCharacter = char_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse character: {}", e))?;

    let corp_url = format!(
        "https://esi.evetech.net/latest/corporations/{}/?datasource=tranquility",
        esi_char.corporation_id
    );
    let (corp_name, corp_ticker) = match client.get(&corp_url).send().await {
        Ok(resp) => resp
            .json::<EsiCorporation>()
            .await
            .ok()
            .map(|c| (Some(c.name), Some(c.ticker)))
            .unwrap_or((None, None)),
        Err(e) => {
            warn!("Failed to fetch corporation {}: {}", esi_char.corporation_id, e);
            (None, None)
        },
    };

    let (alliance_name, alliance_ticker) = if let Some(alliance_id) = esi_char.alliance_id {
        let alliance_url = format!(
            "https://esi.evetech.net/latest/alliances/{}/?datasource=tranquility",
            alliance_id
        );
        match client.get(&alliance_url).send().await {
            Ok(resp) => resp
                .json::<EsiAlliance>()
                .await
                .ok()
                .map(|a| (Some(a.name), Some(a.ticker)))
                .unwrap_or((None, None)),
            Err(e) => {
                warn!("Failed to fetch alliance {}: {}", alliance_id, e);
                (None, None)
            },
        }
    } else {
        (None, None)
    };

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

    let options = Some(tauri_plugin_cache::SetItemOptions {
        ttl: Some(ttl_secs),
        compress: None,
        compression_method: None,
    });
    
    if let Err(e) = cache.set(cache_key.clone(), serde_json::to_value(&info).unwrap(), options) {
        warn!("Failed to cache character {}: {}", character_id, e);
    } else {
        debug!("Cached character {} for {}s", character_id, ttl_secs);
    }

    Ok(info)
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
