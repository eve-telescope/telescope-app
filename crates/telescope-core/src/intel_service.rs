//! Intel network service. All state mutation goes through the pure
//! `domain::intel_reducer`. This layer only does I/O: telescope API calls,
//! persistence, and publishing the new state on a watch channel.

use std::path::PathBuf;

use log::{info, warn};
use reqwest::Client;
use tokio::sync::{Mutex, watch};

use crate::domain::intel_reducer::{IntelAction, reduce};
use crate::intel_state::IntelState;
use crate::models::*;
use crate::telescope_api::{self, TelescopeClient};

/// Fields of an annotation as sent to the API on create and update.
#[derive(Clone, Debug, Default)]
pub struct EntryInput {
    pub entity_type: String,
    pub entity_id: i64,
    pub entity_name: String,
    pub color: String,
    pub label: String,
    pub notes: Option<String>,
}

/// Fields of a network access grant.
#[derive(Clone, Debug)]
pub struct AccessInput {
    pub accessible_type: String,
    pub accessible_id: i64,
    pub accessible_name: String,
    pub permission: String,
}

pub struct IntelService {
    state: Mutex<IntelState>,
    clients: TelescopeClient,
    app_dir: PathBuf,
    changes: watch::Sender<IntelState>,
}

impl IntelService {
    pub fn new(app_dir: PathBuf, api_base_url: String) -> Self {
        let initial = reduce(
            IntelState::load(&app_dir),
            IntelAction::SetBaseUrl(api_base_url),
        );
        let (changes, _) = watch::channel(initial.clone());
        Self {
            state: Mutex::new(initial),
            clients: TelescopeClient::default(),
            app_dir,
            changes,
        }
    }

    /// Receives the full state after every change.
    pub fn subscribe(&self) -> watch::Receiver<IntelState> {
        self.changes.subscribe()
    }

    pub async fn snapshot(&self) -> IntelState {
        self.state.lock().await.clone()
    }

    async fn apply(&self, action: IntelAction, persist: bool) {
        let mut s = self.state.lock().await;
        *s = reduce(std::mem::take(&mut *s), action);
        if persist {
            s.save(&self.app_dir);
        }
        self.changes.send_replace(s.clone());
    }

    /// Returns (base_url, authorized client), reusing the cached client
    /// unless the token changed.
    async fn api_context(&self) -> Result<(String, Client), String> {
        let (base_url, token) = {
            let s = self.state.lock().await;
            let token = s.api_token.clone().ok_or("Not authenticated")?;
            (s.api_base_url.clone(), token)
        };
        let client = self.clients.for_token(&token)?;
        Ok((base_url, client))
    }

    async fn is_selected(&self, network_id: i64) -> bool {
        self.state
            .lock()
            .await
            .selected_network
            .as_ref()
            .map(|n| n.id)
            == Some(network_id)
    }

    async fn network_name(&self, network_id: i64) -> String {
        self.state
            .lock()
            .await
            .networks
            .iter()
            .find(|n| n.id == network_id)
            .map(|n| n.name.clone())
            .unwrap_or_default()
    }

    /// Re-fetches the selected network detail from the API and updates state.
    /// Failures are logged rather than surfaced: the local state was already
    /// updated optimistically and the next refetch will reconcile.
    async fn refetch_selected_network(&self, client: &Client, base_url: &str, network_id: i64) {
        match telescope_api::get_network_detail(client, base_url, network_id).await {
            Ok(detail) => self.apply(IntelAction::SelectNetwork(detail), false).await,
            Err(err) => warn!("[Intel] Failed to refresh network {}: {}", network_id, err),
        }
    }

    pub async fn set_api_token(&self, token: String) -> Result<(), String> {
        self.apply(IntelAction::SetToken(token), true).await;
        self.fetch_networks().await
    }

    pub async fn logout(&self) {
        self.apply(IntelAction::Logout, true).await;
    }

    pub async fn set_active_network_ids(&self, ids: Vec<i64>) {
        self.apply(IntelAction::SetActiveNetworkIds(ids), true)
            .await;
    }

    pub async fn fetch_networks(&self) -> Result<(), String> {
        let (base_url, client) = self.api_context().await?;
        let networks = telescope_api::fetch_networks(&client, &base_url).await?;
        info!("[Intel] Loaded {} networks", networks.len());
        self.apply(IntelAction::SetNetworks(networks), false).await;
        Ok(())
    }

    pub async fn create_network(&self, name: &str) -> Result<IntelNetwork, String> {
        let (base_url, client) = self.api_context().await?;
        let network = telescope_api::create_network(&client, &base_url, name).await?;
        self.apply(IntelAction::AddNetwork(network.clone()), false)
            .await;
        Ok(network)
    }

    pub async fn delete_network(&self, network_id: i64) -> Result<(), String> {
        let (base_url, client) = self.api_context().await?;
        telescope_api::delete_network(&client, &base_url, network_id).await?;
        self.apply(IntelAction::RemoveNetwork(network_id), false)
            .await;
        Ok(())
    }

    pub async fn select_network(&self, network_id: i64) -> Result<NetworkDetail, String> {
        let (base_url, client) = self.api_context().await?;
        let detail = telescope_api::get_network_detail(&client, &base_url, network_id).await?;
        self.apply(IntelAction::SelectNetwork(detail.clone()), false)
            .await;
        Ok(detail)
    }

    pub async fn clear_selected_network(&self) {
        self.apply(IntelAction::ClearSelected, false).await;
    }

    /// Drops looked-up entries before a new scan so stale annotations don't
    /// show against the new pilots.
    pub async fn clear_entries(&self) {
        self.apply(IntelAction::SetEntries(Vec::new()), false).await;
    }

    pub async fn lookup_intel(&self, entity_ids: &[i64]) -> Result<(), String> {
        if entity_ids.is_empty() {
            return Ok(());
        }
        let (base_url, client) = self.api_context().await?;
        let entries = telescope_api::lookup_intel(&client, &base_url, entity_ids).await?;
        info!("[Intel] Lookup returned {} entries", entries.len());
        self.apply(IntelAction::SetEntries(entries), false).await;
        Ok(())
    }

    pub async fn add_entry(
        &self,
        network_id: i64,
        input: EntryInput,
    ) -> Result<IntelEntry, String> {
        let (base_url, client) = self.api_context().await?;
        let mut entry = telescope_api::add_intel_entry(
            &client,
            &base_url,
            network_id,
            &input.entity_type,
            input.entity_id,
            &input.entity_name,
            &input.color,
            &input.label,
            input.notes.as_deref(),
        )
        .await?;
        entry.network_name = self.network_name(network_id).await;
        self.after_entry_change(
            IntelAction::UpsertEntry(entry.clone()),
            &client,
            &base_url,
            network_id,
        )
        .await;
        Ok(entry)
    }

    pub async fn update_entry(
        &self,
        network_id: i64,
        entry_id: i64,
        input: EntryInput,
    ) -> Result<IntelEntry, String> {
        let (base_url, client) = self.api_context().await?;
        let mut entry = telescope_api::update_intel_entry(
            &client,
            &base_url,
            network_id,
            entry_id,
            &input.entity_type,
            input.entity_id,
            &input.entity_name,
            &input.color,
            &input.label,
            input.notes.as_deref(),
        )
        .await?;
        entry.network_name = self.network_name(network_id).await;
        self.after_entry_change(
            IntelAction::UpsertEntry(entry.clone()),
            &client,
            &base_url,
            network_id,
        )
        .await;
        Ok(entry)
    }

    pub async fn remove_entry(&self, network_id: i64, entry_id: i64) -> Result<(), String> {
        let (base_url, client) = self.api_context().await?;
        telescope_api::remove_intel_entry(&client, &base_url, network_id, entry_id).await?;
        self.after_entry_change(
            IntelAction::RemoveEntry(entry_id),
            &client,
            &base_url,
            network_id,
        )
        .await;
        Ok(())
    }

    /// Applies a realtime "entry deleted" notification without an API call.
    pub async fn forget_entry(&self, entry_id: i64) {
        self.apply(IntelAction::RemoveEntry(entry_id), false).await;
    }

    async fn after_entry_change(
        &self,
        action: IntelAction,
        client: &Client,
        base_url: &str,
        network_id: i64,
    ) {
        self.apply(action, false).await;
        if self.is_selected(network_id).await {
            self.refetch_selected_network(client, base_url, network_id)
                .await;
        }
    }

    pub async fn add_access(
        &self,
        network_id: i64,
        input: AccessInput,
    ) -> Result<NetworkAccess, String> {
        let (base_url, client) = self.api_context().await?;
        let access = telescope_api::add_network_access(
            &client,
            &base_url,
            network_id,
            &input.accessible_type,
            input.accessible_id,
            &input.accessible_name,
            &input.permission,
        )
        .await?;
        self.refetch_selected_network(&client, &base_url, network_id)
            .await;
        Ok(access)
    }

    pub async fn remove_access(&self, network_id: i64, access_id: i64) -> Result<(), String> {
        let (base_url, client) = self.api_context().await?;
        telescope_api::remove_network_access(&client, &base_url, network_id, access_id).await?;
        self.refetch_selected_network(&client, &base_url, network_id)
            .await;
        Ok(())
    }

    pub async fn share_scan(
        &self,
        network_id: i64,
        scan_type: &str,
        raw_text: &str,
        solar_system: Option<&str>,
    ) -> Result<NetworkScan, String> {
        let (base_url, client) = self.api_context().await?;
        telescope_api::share_scan(
            &client,
            &base_url,
            network_id,
            scan_type,
            raw_text,
            solar_system,
        )
        .await
    }

    pub async fn fetch_scans(&self, network_id: i64, page: i64) -> Result<PaginatedScans, String> {
        let (base_url, client) = self.api_context().await?;
        telescope_api::fetch_scans(&client, &base_url, network_id, page).await
    }

    pub async fn fetch_scan(&self, network_id: i64, scan_id: i64) -> Result<NetworkScan, String> {
        let (base_url, client) = self.api_context().await?;
        telescope_api::fetch_scan(&client, &base_url, network_id, scan_id).await
    }

    pub async fn search_entities(
        &self,
        query: &str,
        category: Option<&str>,
    ) -> Result<Vec<SearchResult>, String> {
        let (base_url, client) = self.api_context().await?;
        telescope_api::search_entities(&client, &base_url, query, category).await
    }
}
