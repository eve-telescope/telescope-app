use futures::future::join_all;
use log::{info, warn, debug, error};
use tauri::AppHandle;

use crate::api::{create_client, esi, zkill};
use crate::intel::calculate_threat_level;
use crate::models::{CharacterInfo, PilotIntel};

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

    info!("Looking up {} pilots", names.len());
    debug!("Pilot names: {:?}", names);

    let id_map = esi::resolve_character_ids(&client, &names).await?;
    info!("Resolved {} character IDs", id_map.len());

    let tasks: Vec<_> = names
        .into_iter()
        .map(|name| {
            let app = app.clone();
            let client = client.clone();
            let character_id = id_map.get(&name.to_lowercase()).copied();
            async move { fetch_pilot_intel(&app, &client, name, character_id).await }
        })
        .collect();

    let mut results: Vec<PilotIntel> = join_all(tasks).await;

    let threats: Vec<_> = results.iter()
        .filter(|p| p.threat_level == "EXTREME" || p.threat_level == "HIGH")
        .map(|p| p.character.name.as_str())
        .collect();
    
    if !threats.is_empty() {
        warn!("High threat pilots detected: {:?}", threats);
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

    info!("Lookup complete, returning {} results", results.len());
    Ok(results)
}

async fn fetch_pilot_intel(
    app: &AppHandle,
    client: &reqwest::Client,
    name: String,
    character_id: Option<i64>,
) -> PilotIntel {
    match character_id {
        Some(id) => match esi::fetch_character_info(app, client, id).await {
            Ok(character) => {
                debug!("Fetched ESI info for {} (ID: {})", character.name, id);
                
                let zkill = match zkill::fetch_stats(app, client, id).await {
                    Ok(stats) => {
                        debug!("Fetched zKill stats for {} - {} kills, {} losses", 
                            character.name, stats.ships_destroyed, stats.ships_lost);
                        Some(stats)
                    }
                    Err(e) => {
                        warn!("Failed to fetch zKill stats for {} (ID: {}): {}", character.name, id, e);
                        None
                    }
                };
                
                let threat_level = calculate_threat_level(&zkill);
                PilotIntel {
                    character,
                    zkill,
                    threat_level,
                    error: None,
                }
            }
            Err(e) => {
                error!("Failed to fetch ESI info for {} (ID: {}): {}", name, id, e);
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
                }
            },
        },
        None => {
            warn!("Character not found in ESI: {}", name);
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
            }
        },
    }
}
