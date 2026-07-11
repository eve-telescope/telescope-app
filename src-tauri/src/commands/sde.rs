//! SDE index commands: thin wrappers over the `crate::sde` service (I/O)
//! and `crate::domain::dscan` (pure parsing).

use crate::models::{DscanParseResult, SdeStatus};
use crate::sde;

#[tauri::command]
pub async fn ensure_sde_index(
    app_dir: tauri::State<'_, std::path::PathBuf>,
    sde_service: tauri::State<'_, sde::SdeService>,
) -> Result<SdeStatus, String> {
    sde::ensure_sde_index(app_dir.inner().as_path(), sde_service.inner()).await
}

#[tauri::command]
pub async fn get_sde_status(
    app_dir: tauri::State<'_, std::path::PathBuf>,
) -> Result<SdeStatus, String> {
    sde::get_sde_status(app_dir.inner().as_path()).await
}

#[tauri::command]
pub async fn parse_dscan(
    app_dir: tauri::State<'_, std::path::PathBuf>,
    sde_service: tauri::State<'_, sde::SdeService>,
    text: String,
) -> Result<DscanParseResult, String> {
    let index = sde_service
        .index(app_dir.inner().as_path())
        .await?
        .ok_or_else(|| "SDE index is not ready yet".to_string())?;

    // Large pastes are pure CPU work; keep them off the async runtime.
    tokio::task::spawn_blocking(move || crate::domain::dscan::parse_dscan_text(&index, &text))
        .await
        .map_err(|err| err.to_string())
}
