use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::models::{IntelEntry, IntelNetwork, NetworkDetail};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IntelState {
    pub api_base_url: String,
    pub api_token: Option<String>,
    pub networks: Vec<IntelNetwork>,
    pub entries: Vec<IntelEntry>,
    pub selected_network: Option<NetworkDetail>,
    pub active_network_ids: Vec<i64>,
}

impl Default for IntelState {
    fn default() -> Self {
        Self {
            api_base_url: "https://eve-telescope.com".to_string(),
            api_token: None,
            networks: Vec::new(),
            entries: Vec::new(),
            selected_network: None,
            active_network_ids: Vec::new(),
        }
    }
}

/// Persisted subset of IntelState (only what survives restarts).
#[derive(Debug, Serialize, Deserialize)]
struct PersistedState {
    api_token: Option<String>,
    active_network_ids: Vec<i64>,
}

impl IntelState {
    pub fn save(&self, app_dir: &PathBuf) {
        let path = app_dir.join("intel_state.json");
        let persisted = PersistedState {
            api_token: self.api_token.clone(),
            active_network_ids: self.active_network_ids.clone(),
        };
        if let Ok(json) = serde_json::to_string(&persisted) {
            let _ = fs::write(path, json);
        }
    }

    pub fn load(app_dir: &PathBuf) -> Self {
        let path = app_dir.join("intel_state.json");
        if let Ok(json) = fs::read_to_string(path) {
            if let Ok(persisted) = serde_json::from_str::<PersistedState>(&json) {
                return Self {
                    api_token: persisted.api_token,
                    active_network_ids: persisted.active_network_ids,
                    ..Self::default()
                };
            }
        }
        Self::default()
    }
}
