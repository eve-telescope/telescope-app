//! Deep-link dispatch: routing parsed telescope:// links into the app
//! (applying auth tokens, forwarding/parking share codes). Parsing itself
//! is pure and lives in `crate::domain::deeplink`.

use log::{error, info, warn};
use tauri::{AppHandle, Emitter, Manager};

use crate::domain::deeplink::{parse, DeepLink};

/// Event carrying a share code to the frontend (payload: the code string).
pub const SHARE_EVENT: &str = "deep-link-share";

/// Share code from a deep link that arrived before the frontend mounted
/// (cold start via link click); the frontend collects it once on startup.
#[derive(Default)]
pub struct PendingShare(std::sync::Mutex<Option<String>>);

#[tauri::command]
pub fn take_pending_deep_link_share(state: tauri::State<'_, PendingShare>) -> Option<String> {
    state.0.lock().ok().and_then(|mut guard| guard.take())
}

/// Handles a deep link received while the app is running: auth tokens are
/// applied entirely backend-side; share codes are forwarded to the frontend.
pub fn handle_runtime(app: &AppHandle, url: &str) {
    info!("[DeepLink] Runtime URL received: {}", url);
    match parse(url) {
        Some(DeepLink::Auth { token }) => spawn_apply_token(app, token),
        Some(DeepLink::Share { code }) => {
            info!("[DeepLink] Forwarding share code to frontend: {}", code);
            let _ = app.emit(SHARE_EVENT, code);
        }
        None => warn!("[DeepLink] Ignoring unrecognized URL: {}", url),
    }
}

/// Handles deep links present at startup, before the webview exists: auth
/// applies immediately; share codes are parked until the frontend asks.
pub fn handle_startup(app: &AppHandle, url: &str) {
    info!("[DeepLink] Startup URL received: {}", url);
    match parse(url) {
        Some(DeepLink::Auth { token }) => spawn_apply_token(app, token),
        Some(DeepLink::Share { code }) => {
            if let Ok(mut pending) = app.state::<PendingShare>().0.lock() {
                *pending = Some(code);
            }
        }
        None => warn!("[DeepLink] Ignoring unrecognized URL: {}", url),
    }
}

fn spawn_apply_token(app: &AppHandle, token: String) {
    info!("[DeepLink] Applying auth token from deep link");
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        match crate::intel_commands::apply_api_token(&app, token).await {
            Ok(()) => info!("[DeepLink] Auth token applied"),
            Err(e) => error!("[DeepLink] Failed to apply auth token: {}", e),
        }
    });
}
