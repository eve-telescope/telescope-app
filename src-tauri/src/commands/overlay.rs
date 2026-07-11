//! Overlay window management.
//!
//! Deliberately no state machine here: the OS window handle is the only
//! state (present/absent), so tauri's window registry already *is* the
//! machine — a mirror enum could only drift from it.

use log::{error, info};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

#[tauri::command]
pub fn is_overlay_open(app: AppHandle) -> bool {
    app.get_webview_window("overlay").is_some()
}

#[tauri::command]
pub async fn open_overlay(app: AppHandle) -> Result<bool, String> {
    info!("open_overlay: starting");

    if let Some(window) = app.get_webview_window("overlay") {
        info!("open_overlay: window exists, focusing");
        let _ = window.set_focus();
        return Ok(true);
    }

    info!("open_overlay: building window");

    let _window = WebviewWindowBuilder::new(&app, "overlay", WebviewUrl::App("/overlay".into()))
        .title("Telescope Overlay")
        .inner_size(580.0, 450.0)
        .min_inner_size(480.0, 200.0)
        .decorations(false)
        .always_on_top(true)
        .visible(false)
        .build()
        .map_err(|e| {
            error!("open_overlay: build failed: {}", e);
            format!("Failed to create overlay window: {}", e)
        })?;

    info!("open_overlay: window built");
    Ok(true)
}

#[tauri::command]
pub async fn close_overlay(app: AppHandle) -> Result<bool, String> {
    if let Some(window) = app.get_webview_window("overlay") {
        window
            .close()
            .map_err(|e| format!("Failed to close overlay: {}", e))?;

        // Notify main window that overlay was closed
        let _ = app.emit("overlay-closed", ());

        info!("Overlay window closed");
        Ok(false)
    } else {
        Ok(false)
    }
}

#[tauri::command]
pub async fn toggle_overlay(app: AppHandle) -> Result<bool, String> {
    if app.get_webview_window("overlay").is_some() {
        close_overlay(app).await
    } else {
        open_overlay(app).await
    }
}
