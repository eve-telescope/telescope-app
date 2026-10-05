//! Update check against the latest GitHub release.

use log::{info, warn};

use crate::api::create_client;
use crate::domain::version::is_newer_version;

#[derive(Clone, Debug)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub release_url: String,
    pub release_notes: String,
}

pub async fn check_for_update() -> Result<Option<UpdateInfo>, String> {
    let client = create_client()?;
    let current_version = env!("CARGO_PKG_VERSION");

    info!("Checking for updates (current: v{})", current_version);

    let response = client
        .get("https://api.github.com/repos/eve-telescope/telescope-app/releases/latest")
        .send()
        .await
        .map_err(|e| {
            warn!("Failed to check for updates: {}", e);
            format!("Failed to check for updates: {}", e)
        })?;

    if !response.status().is_success() {
        warn!("Update check failed: HTTP {}", response.status());
        return Ok(None);
    }

    let release: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse release: {}", e))?;

    let latest_version = release["tag_name"]
        .as_str()
        .unwrap_or("")
        .trim_start_matches('v');

    if is_newer_version(latest_version, current_version) {
        info!(
            "Update available: v{} → v{}",
            current_version, latest_version
        );
        Ok(Some(UpdateInfo {
            current_version: current_version.to_string(),
            latest_version: latest_version.to_string(),
            release_url: release["html_url"].as_str().unwrap_or("").to_string(),
            release_notes: release["body"].as_str().unwrap_or("").to_string(),
        }))
    } else {
        info!("App is up to date (v{})", current_version);
        Ok(None)
    }
}
