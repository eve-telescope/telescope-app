//! Intel network commands. All state mutation goes through the pure
//! `domain::intel_reducer`. This layer only does I/O: telescope API calls,
//! persistence, and event emission.

use log::{info, warn};
use reqwest::Client;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::Mutex;

use crate::domain::intel_reducer::{reduce, IntelAction};
use crate::intel_state::IntelState;
use crate::models::*;
use crate::telescope_api::{self, TelescopeClient};

/// Applies an action to the locked state via the pure reducer.
fn apply(state: &mut IntelState, action: IntelAction) {
    *state = reduce(std::mem::take(state), action);
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

/// Returns (base_url, authorized client) from locked state, reusing the
/// cached client unless the token changed.
async fn api_context(
    state: &Mutex<IntelState>,
    clients: &TelescopeClient,
) -> Result<(String, Client), String> {
    let (base_url, token) = {
        let s = state.lock().await;
        let token = s.api_token.clone().ok_or("Not authenticated")?;
        (s.api_base_url.clone(), token)
    };
    let client = clients.for_token(&token)?;
    Ok((base_url, client))
}

/// Re-fetches the selected network detail from the API and updates state.
/// Failures are logged rather than surfaced: the local state was already
/// updated optimistically and the next refetch will reconcile.
async fn refetch_selected_network(
    app: &AppHandle,
    state: &Mutex<IntelState>,
    client: &Client,
    base_url: &str,
    network_id: i64,
) {
    match telescope_api::get_network_detail(client, base_url, network_id).await {
        Ok(detail) => {
            let mut s = state.lock().await;
            apply(&mut s, IntelAction::SelectNetwork(detail));
            emit_state(app, &s);
        }
        Err(err) => warn!("[Intel] Failed to refresh network {}: {}", network_id, err),
    }
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
    apply(&mut s, IntelAction::SetBaseUrl(url));
    emit_state(&app, &s);
    Ok(())
}

#[tauri::command]
pub async fn set_api_token(
    app: AppHandle,
    state: State<'_, Mutex<IntelState>>,
    clients: State<'_, TelescopeClient>,
    token: String,
) -> Result<(), String> {
    {
        let mut s = state.lock().await;
        apply(&mut s, IntelAction::SetToken(token));
        persist_state(&app, &s);
        emit_state(&app, &s);
    }
    // Refresh networks with the new token, after the state lock is released.
    fetch_networks(app, state, clients).await
}

/// Applies an auth token exactly like the `set_api_token` command, but
/// callable from Rust-side handlers (e.g. deep links) where `State` guards
/// come from the `AppHandle` instead of an invoke.
pub async fn apply_api_token(app: &AppHandle, token: String) -> Result<(), String> {
    use tauri::Manager;
    let state = app.state::<Mutex<IntelState>>();
    let clients = app.state::<TelescopeClient>();
    set_api_token(app.clone(), state, clients, token).await
}

#[tauri::command]
pub async fn logout_intel(
    app: AppHandle,
    state: State<'_, Mutex<IntelState>>,
) -> Result<(), String> {
    let mut s = state.lock().await;
    apply(&mut s, IntelAction::Logout);
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
    apply(&mut s, IntelAction::SetActiveNetworkIds(ids));
    persist_state(&app, &s);
    emit_state(&app, &s);
    Ok(())
}

#[tauri::command]
pub async fn fetch_networks(
    app: AppHandle,
    state: State<'_, Mutex<IntelState>>,
    clients: State<'_, TelescopeClient>,
) -> Result<(), String> {
    let (base_url, client) = api_context(&state, &clients).await?;

    let networks = telescope_api::fetch_networks(&client, &base_url).await?;
    info!("[Intel] Loaded {} networks", networks.len());

    let mut s = state.lock().await;
    apply(&mut s, IntelAction::SetNetworks(networks));
    emit_state(&app, &s);
    Ok(())
}

#[tauri::command]
pub async fn create_network(
    app: AppHandle,
    state: State<'_, Mutex<IntelState>>,
    clients: State<'_, TelescopeClient>,
    name: String,
) -> Result<IntelNetwork, String> {
    let (base_url, client) = api_context(&state, &clients).await?;

    let network = telescope_api::create_network(&client, &base_url, &name).await?;

    let mut s = state.lock().await;
    apply(&mut s, IntelAction::AddNetwork(network.clone()));
    emit_state(&app, &s);
    Ok(network)
}

#[tauri::command]
pub async fn delete_network(
    app: AppHandle,
    state: State<'_, Mutex<IntelState>>,
    clients: State<'_, TelescopeClient>,
    network_id: i64,
) -> Result<(), String> {
    let (base_url, client) = api_context(&state, &clients).await?;

    telescope_api::delete_network(&client, &base_url, network_id).await?;

    let mut s = state.lock().await;
    apply(&mut s, IntelAction::RemoveNetwork(network_id));
    emit_state(&app, &s);
    Ok(())
}

#[tauri::command]
pub async fn select_network(
    app: AppHandle,
    state: State<'_, Mutex<IntelState>>,
    clients: State<'_, TelescopeClient>,
    network_id: i64,
) -> Result<NetworkDetail, String> {
    let (base_url, client) = api_context(&state, &clients).await?;

    let detail = telescope_api::get_network_detail(&client, &base_url, network_id).await?;

    let mut s = state.lock().await;
    apply(&mut s, IntelAction::SelectNetwork(detail.clone()));
    emit_state(&app, &s);
    Ok(detail)
}

#[tauri::command]
pub async fn clear_selected_network(
    app: AppHandle,
    state: State<'_, Mutex<IntelState>>,
) -> Result<(), String> {
    let mut s = state.lock().await;
    apply(&mut s, IntelAction::ClearSelected);
    emit_state(&app, &s);
    Ok(())
}

#[tauri::command]
pub async fn lookup_intel(
    app: AppHandle,
    state: State<'_, Mutex<IntelState>>,
    clients: State<'_, TelescopeClient>,
    entity_ids: Vec<i64>,
) -> Result<(), String> {
    if entity_ids.is_empty() {
        return Ok(());
    }

    let (base_url, client) = api_context(&state, &clients).await?;

    let entries = telescope_api::lookup_intel(&client, &base_url, &entity_ids).await?;
    info!("[Intel] Lookup returned {} entries", entries.len());

    let mut s = state.lock().await;
    apply(&mut s, IntelAction::SetEntries(entries));
    emit_state(&app, &s);
    Ok(())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn add_intel_entry(
    app: AppHandle,
    state: State<'_, Mutex<IntelState>>,
    clients: State<'_, TelescopeClient>,
    network_id: i64,
    entity_type: String,
    entity_id: i64,
    entity_name: String,
    color: String,
    label: String,
    notes: Option<String>,
) -> Result<IntelEntry, String> {
    let (base_url, client) = api_context(&state, &clients).await?;
    let network_name = {
        let s = state.lock().await;
        s.networks
            .iter()
            .find(|n| n.id == network_id)
            .map(|n| n.name.clone())
            .unwrap_or_default()
    };

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
    apply(&mut s, IntelAction::UpsertEntry(entry.clone()));

    if s.selected_network.as_ref().map(|n| n.id) == Some(network_id) {
        drop(s);
        refetch_selected_network(&app, &state, &client, &base_url, network_id).await;
    } else {
        emit_state(&app, &s);
    }

    Ok(entry)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn update_intel_entry(
    app: AppHandle,
    state: State<'_, Mutex<IntelState>>,
    clients: State<'_, TelescopeClient>,
    network_id: i64,
    entry_id: i64,
    entity_type: String,
    entity_id: i64,
    entity_name: String,
    color: String,
    label: String,
    notes: Option<String>,
) -> Result<IntelEntry, String> {
    let (base_url, client) = api_context(&state, &clients).await?;
    let network_name = {
        let s = state.lock().await;
        s.networks
            .iter()
            .find(|n| n.id == network_id)
            .map(|n| n.name.clone())
            .unwrap_or_default()
    };

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
    apply(&mut s, IntelAction::UpsertEntry(entry.clone()));

    if s.selected_network.as_ref().map(|n| n.id) == Some(network_id) {
        drop(s);
        refetch_selected_network(&app, &state, &client, &base_url, network_id).await;
    } else {
        emit_state(&app, &s);
    }

    Ok(entry)
}

#[tauri::command]
pub async fn remove_intel_entry(
    app: AppHandle,
    state: State<'_, Mutex<IntelState>>,
    clients: State<'_, TelescopeClient>,
    network_id: i64,
    entry_id: i64,
) -> Result<(), String> {
    let (base_url, client) = api_context(&state, &clients).await?;

    telescope_api::remove_intel_entry(&client, &base_url, network_id, entry_id).await?;

    let mut s = state.lock().await;
    apply(&mut s, IntelAction::RemoveEntry(entry_id));

    if s.selected_network.as_ref().map(|n| n.id) == Some(network_id) {
        drop(s);
        refetch_selected_network(&app, &state, &client, &base_url, network_id).await;
    } else {
        emit_state(&app, &s);
    }
    Ok(())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn add_network_access(
    app: AppHandle,
    state: State<'_, Mutex<IntelState>>,
    clients: State<'_, TelescopeClient>,
    network_id: i64,
    accessible_type: String,
    accessible_id: i64,
    accessible_name: String,
    permission: String,
) -> Result<NetworkAccess, String> {
    let (base_url, client) = api_context(&state, &clients).await?;

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

    refetch_selected_network(&app, &state, &client, &base_url, network_id).await;
    Ok(access)
}

#[tauri::command]
pub async fn remove_network_access(
    app: AppHandle,
    state: State<'_, Mutex<IntelState>>,
    clients: State<'_, TelescopeClient>,
    network_id: i64,
    access_id: i64,
) -> Result<(), String> {
    let (base_url, client) = api_context(&state, &clients).await?;

    telescope_api::remove_network_access(&client, &base_url, network_id, access_id).await?;

    refetch_selected_network(&app, &state, &client, &base_url, network_id).await;
    Ok(())
}

#[tauri::command]
pub async fn share_scan(
    state: State<'_, Mutex<IntelState>>,
    clients: State<'_, TelescopeClient>,
    network_id: i64,
    scan_type: String,
    raw_text: String,
    solar_system: Option<String>,
) -> Result<NetworkScan, String> {
    let (base_url, client) = api_context(&state, &clients).await?;

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
    clients: State<'_, TelescopeClient>,
    network_id: i64,
    page: Option<i64>,
) -> Result<PaginatedScans, String> {
    let (base_url, client) = api_context(&state, &clients).await?;

    telescope_api::fetch_scans(&client, &base_url, network_id, page.unwrap_or(1)).await
}

#[tauri::command]
pub async fn fetch_network_scan(
    state: State<'_, Mutex<IntelState>>,
    clients: State<'_, TelescopeClient>,
    network_id: i64,
    scan_id: i64,
) -> Result<NetworkScan, String> {
    let (base_url, client) = api_context(&state, &clients).await?;

    telescope_api::fetch_scan(&client, &base_url, network_id, scan_id).await
}

#[tauri::command]
pub async fn search_entities(
    state: State<'_, Mutex<IntelState>>,
    clients: State<'_, TelescopeClient>,
    query: String,
    category: Option<String>,
) -> Result<Vec<SearchResult>, String> {
    let (base_url, client) = api_context(&state, &clients).await?;

    telescope_api::search_entities(&client, &base_url, &query, category.as_deref()).await
}
