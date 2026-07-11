//! System-level commands: cache clearing, update checks, external links.

use log::{error, info, warn};
use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_cache::CacheExt;

use crate::api::create_client;
use crate::domain::version::is_newer_version;

#[tauri::command]
pub fn clear_cache(app: AppHandle) -> Result<(), String> {
    let cache = app.cache();
    cache
        .clear()
        .map_err(|e| format!("Failed to clear cache: {}", e))?;
    info!("Cache cleared");
    Ok(())
}

#[derive(Clone, Serialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub release_url: String,
    pub release_notes: String,
}

#[tauri::command]
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

/// Opens an http(s) URL in the default browser. Routed through Rust (instead
/// of the opener plugin's JS API) so every attempt and failure is logged.
#[tauri::command]
pub async fn open_external(app: AppHandle, url: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;

    if !url.starts_with("https://") && !url.starts_with("http://") {
        return Err(format!("Refusing to open non-http(s) URL: {}", url));
    }
    info!("[Opener] Opening external URL: {}", url);

    // On Linux the app is often launched with LD_PRELOAD workarounds for
    // WebKitGTK/GPU issues. Spawned children inherit that environment and
    // can crash on startup — a silently dead xdg-open means the browser
    // never appears — so launch the opener with LD_PRELOAD stripped.
    #[cfg(target_os = "linux")]
    {
        match std::process::Command::new("xdg-open")
            .arg(&url)
            .env_remove("LD_PRELOAD")
            .spawn()
        {
            Ok(_) => {
                info!("[Opener] xdg-open spawned successfully");
                return Ok(());
            }
            Err(e) => {
                // The plugin fallback can't strip the environment, so on
                // setups that need the LD_PRELOAD workaround it may inherit
                // the same breakage — log enough to tell the cases apart.
                warn!(
                    "[Opener] xdg-open unavailable ({}), falling back to opener plugin{}",
                    e,
                    if std::env::var_os("LD_PRELOAD").is_some() {
                        " (LD_PRELOAD is set and will be inherited)"
                    } else {
                        ""
                    }
                );
            }
        }
    }

    match app.opener().open_url(&url, None::<&str>) {
        Ok(()) => {
            info!("[Opener] Browser launch requested successfully");
            Ok(())
        }
        Err(e) => {
            error!("[Opener] Failed to open URL: {}", e);
            Err(e.to_string())
        }
    }
}
