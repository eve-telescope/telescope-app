use log::info;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::Mutex;

use crate::intel_state::IntelState;
use crate::models::*;
use crate::telescope_api;

fn upsert_scan_entry(entries: &mut Vec<IntelEntry>, entry: &IntelEntry) {
    if let Some(idx) = entries.iter().position(|e| e.id == entry.id) {
        entries[idx] = entry.clone();
    } else {
        entries.push(entry.clone());
    }
}

fn emit_state(app: &AppHandle, state: &IntelState) {
    let _ = app.emit("intel-state-changed", state);
}

fn persist_state(app: &AppHandle, state: &IntelState) {
    use std::path::PathBuf;
    use tauri::Manager;
    if let Some(app_dir) = app.try_state::<PathBuf>() {
        state.save(&app_dir);
    }
}

/// Returns (base_url, token) from locked state.
async fn api_config(state: &Mutex<IntelState>) -> Result<(String, String), String> {
    let s = state.lock().await;
    let token = s.api_token.clone().ok_or("Not authenticated")?;
    Ok((s.api_base_url.clone(), token))
}

/// Re-fetches the selected network detail from the API and updates state.
async fn refetch_selected_network(
    app: &AppHandle,
    state: &Mutex<IntelState>,
    base_url: &str,
    token: &str,
    network_id: i64,
) -> Result<(), String> {
    let client = telescope_api::build_client(token)?;
    if let Ok(detail) = telescope_api::get_network_detail(&client, base_url, network_id).await {
        let mut s = state.lock().await;
        s.selected_network = Some(detail);
        emit_state(app, &s);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn get_intel_state(state: State<'_, Mutex<IntelState>>) -> Result<IntelState, String> {
    Ok(state.lock().await.clone())
}

#[tauri::command]
pub async fn set_api_base_url(
    app: AppHandle,
    state: State<'_, Mutex<IntelState>>,
    url: String,
) -> Result<(), String> {
    let mut s = state.lock().await;
    s.api_base_url = url;
    emit_state(&app, &s);
    Ok(())
}

#[tauri::command]
pub async fn set_api_token(
    app: AppHandle,
    state: State<'_, Mutex<IntelState>>,
    token: String,
) -> Result<(), String> {
    {
        let mut s = state.lock().await;
        s.api_token = Some(token);
        persist_state(&app, &s);
        emit_state(&app, &s);
    }
    fetch_networks(app, state).await
}

#[tauri::command]
pub async fn logout_intel(
    app: AppHandle,
    state: State<'_, Mutex<IntelState>>,
) -> Result<(), String> {
    let mut s = state.lock().await;
    let base_url = s.api_base_url.clone();
    *s = IntelState {
        api_base_url: base_url,
        ..IntelState::default()
    };
    persist_state(&app, &s);
    emit_state(&app, &s);
    Ok(())
}

#[tauri::command]
pub async fn set_active_network_ids(
    app: AppHandle,
    state: State<'_, Mutex<IntelState>>,
    ids: Vec<i64>,
) -> Result<(), String> {
    let mut s = state.lock().await;
    s.active_network_ids = ids;
    persist_state(&app, &s);
    emit_state(&app, &s);
    Ok(())
}

#[tauri::command]
pub async fn fetch_networks(
    app: AppHandle,
    state: State<'_, Mutex<IntelState>>,
) -> Result<(), String> {
    let (base_url, token) = api_config(&state).await?;
    let client = telescope_api::build_client(&token)?;

    let networks = telescope_api::fetch_networks(&client, &base_url).await?;
    info!("[Intel] Loaded {} networks", networks.len());

    let mut s = state.lock().await;
    s.networks = networks;
    emit_state(&app, &s);
    Ok(())
}

#[tauri::command]
pub async fn create_network(
    app: AppHandle,
    state: State<'_, Mutex<IntelState>>,
    name: String,
) -> Result<IntelNetwork, String> {
    let (base_url, token) = api_config(&state).await?;
    let client = telescope_api::build_client(&token)?;

    let network = telescope_api::create_network(&client, &base_url, &name).await?;

    let mut s = state.lock().await;
    s.networks.push(network.clone());
    emit_state(&app, &s);
    Ok(network)
}

#[tauri::command]
pub async fn delete_network(
    app: AppHandle,
    state: State<'_, Mutex<IntelState>>,
    network_id: i64,
) -> Result<(), String> {
    let (base_url, token) = api_config(&state).await?;
    let client = telescope_api::build_client(&token)?;

    telescope_api::delete_network(&client, &base_url, network_id).await?;

    let mut s = state.lock().await;
    s.networks.retain(|n| n.id != network_id);
    if s.selected_network.as_ref().map(|n| n.id) == Some(network_id) {
        s.selected_network = None;
    }
    emit_state(&app, &s);
    Ok(())
}

#[tauri::command]
pub async fn select_network(
    app: AppHandle,
    state: State<'_, Mutex<IntelState>>,
    network_id: i64,
) -> Result<NetworkDetail, String> {
    let (base_url, token) = api_config(&state).await?;
    let client = telescope_api::build_client(&token)?;

    let detail = telescope_api::get_network_detail(&client, &base_url, network_id).await?;

    let mut s = state.lock().await;
    s.selected_network = Some(detail.clone());
    emit_state(&app, &s);
    Ok(detail)
}

#[tauri::command]
pub async fn clear_selected_network(
    app: AppHandle,
    state: State<'_, Mutex<IntelState>>,
) -> Result<(), String> {
    let mut s = state.lock().await;
    s.selected_network = None;
    emit_state(&app, &s);
    Ok(())
}

#[tauri::command]
pub async fn lookup_intel(
    app: AppHandle,
    state: State<'_, Mutex<IntelState>>,
    entity_ids: Vec<i64>,
) -> Result<(), String> {
    if entity_ids.is_empty() {
        return Ok(());
    }

    let (base_url, token) = api_config(&state).await?;
    let client = telescope_api::build_client(&token)?;

    let entries = telescope_api::lookup_intel(&client, &base_url, &entity_ids).await?;
    info!("[Intel] Lookup returned {} entries", entries.len());

    let mut s = state.lock().await;
    s.entries = entries;
    emit_state(&app, &s);
    Ok(())
}

#[tauri::command]
pub async fn add_intel_entry(
    app: AppHandle,
    state: State<'_, Mutex<IntelState>>,
    network_id: i64,
    entity_type: String,
    entity_id: i64,
    entity_name: String,
    color: String,
    label: String,
    notes: Option<String>,
) -> Result<IntelEntry, String> {
    let (base_url, token) = api_config(&state).await?;
    let network_name = {
        let s = state.lock().await;
        s.networks
            .iter()
            .find(|n| n.id == network_id)
            .map(|n| n.name.clone())
            .unwrap_or_default()
    };
    let client = telescope_api::build_client(&token)?;

    let mut entry = telescope_api::add_intel_entry(
        &client,
        &base_url,
        network_id,
        &entity_type,
        entity_id,
        &entity_name,
        &color,
        &label,
        notes.as_deref(),
    )
    .await?;
    entry.network_name = network_name;

    let mut s = state.lock().await;
    upsert_scan_entry(&mut s.entries, &entry);

    if s.selected_network.as_ref().map(|n| n.id) == Some(network_id) {
        drop(s);
        let _ = refetch_selected_network(&app, &state, &base_url, &token, network_id).await;
    } else {
        emit_state(&app, &s);
    }

    Ok(entry)
}

#[tauri::command]
pub async fn update_intel_entry(
    app: AppHandle,
    state: State<'_, Mutex<IntelState>>,
    network_id: i64,
    entry_id: i64,
    entity_type: String,
    entity_id: i64,
    entity_name: String,
    color: String,
    label: String,
    notes: Option<String>,
) -> Result<IntelEntry, String> {
    let (base_url, token) = api_config(&state).await?;
    let network_name = {
        let s = state.lock().await;
        s.networks
            .iter()
            .find(|n| n.id == network_id)
            .map(|n| n.name.clone())
            .unwrap_or_default()
    };
    let client = telescope_api::build_client(&token)?;

    let mut entry = telescope_api::update_intel_entry(
        &client,
        &base_url,
        network_id,
        entry_id,
        &entity_type,
        entity_id,
        &entity_name,
        &color,
        &label,
        notes.as_deref(),
    )
    .await?;
    entry.network_name = network_name;

    let mut s = state.lock().await;
    upsert_scan_entry(&mut s.entries, &entry);

    if s.selected_network.as_ref().map(|n| n.id) == Some(network_id) {
        drop(s);
        let _ = refetch_selected_network(&app, &state, &base_url, &token, network_id).await;
    } else {
        emit_state(&app, &s);
    }

    Ok(entry)
}

#[tauri::command]
pub async fn remove_intel_entry(
    app: AppHandle,
    state: State<'_, Mutex<IntelState>>,
    network_id: i64,
    entry_id: i64,
) -> Result<(), String> {
    let (base_url, token) = api_config(&state).await?;
    let client = telescope_api::build_client(&token)?;

    telescope_api::remove_intel_entry(&client, &base_url, network_id, entry_id).await?;

    let mut s = state.lock().await;
    s.entries.retain(|e| e.id != entry_id);

    if s.selected_network.as_ref().map(|n| n.id) == Some(network_id) {
        drop(s);
        let _ = refetch_selected_network(&app, &state, &base_url, &token, network_id).await;
    } else {
        emit_state(&app, &s);
    }
    Ok(())
}

#[tauri::command]
pub async fn add_network_access(
    app: AppHandle,
    state: State<'_, Mutex<IntelState>>,
    network_id: i64,
    accessible_type: String,
    accessible_id: i64,
    accessible_name: String,
    permission: String,
) -> Result<NetworkAccess, String> {
    let (base_url, token) = api_config(&state).await?;
    let client = telescope_api::build_client(&token)?;

    let access = telescope_api::add_network_access(
        &client,
        &base_url,
        network_id,
        &accessible_type,
        accessible_id,
        &accessible_name,
        &permission,
    )
    .await?;

    let _ = refetch_selected_network(&app, &state, &base_url, &token, network_id).await;
    Ok(access)
}

#[tauri::command]
pub async fn remove_network_access(
    app: AppHandle,
    state: State<'_, Mutex<IntelState>>,
    network_id: i64,
    access_id: i64,
) -> Result<(), String> {
    let (base_url, token) = api_config(&state).await?;
    let client = telescope_api::build_client(&token)?;

    telescope_api::remove_network_access(&client, &base_url, network_id, access_id).await?;

    let _ = refetch_selected_network(&app, &state, &base_url, &token, network_id).await;
    Ok(())
}

#[tauri::command]
pub async fn share_scan(
    state: State<'_, Mutex<IntelState>>,
    network_id: i64,
    scan_type: String,
    raw_text: String,
    solar_system: Option<String>,
) -> Result<NetworkScan, String> {
    let (base_url, token) = api_config(&state).await?;
    let client = telescope_api::build_client(&token)?;

    telescope_api::share_scan(
        &client,
        &base_url,
        network_id,
        &scan_type,
        &raw_text,
        solar_system.as_deref(),
    )
    .await
}

#[tauri::command]
pub async fn fetch_network_scans(
    state: State<'_, Mutex<IntelState>>,
    network_id: i64,
    page: Option<i64>,
) -> Result<PaginatedScans, String> {
    let (base_url, token) = api_config(&state).await?;
    let client = telescope_api::build_client(&token)?;

    telescope_api::fetch_scans(&client, &base_url, network_id, page.unwrap_or(1)).await
}

#[tauri::command]
pub async fn fetch_network_scan(
    state: State<'_, Mutex<IntelState>>,
    network_id: i64,
    scan_id: i64,
) -> Result<NetworkScan, String> {
    let (base_url, token) = api_config(&state).await?;
    let client = telescope_api::build_client(&token)?;

    telescope_api::fetch_scan(&client, &base_url, network_id, scan_id).await
}

#[tauri::command]
pub async fn search_entities(
    state: State<'_, Mutex<IntelState>>,
    query: String,
    category: Option<String>,
) -> Result<Vec<SearchResult>, String> {
    let (base_url, token) = api_config(&state).await?;
    let client = telescope_api::build_client(&token)?;

    telescope_api::search_entities(&client, &base_url, &query, category.as_deref()).await
}
