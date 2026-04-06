mod api;
mod commands;
mod intel;
mod intel_commands;
mod intel_state;
mod models;
mod sde;
mod telescope_api;

pub use models::*;

use intel_state::IntelState;
use tauri::Manager;
use tauri_plugin_log::{Target, TargetKind};
use tokio::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // Focus the main window when a second instance is launched
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
                let _ = window.unminimize();
            }
        }))
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::Webview),
                    Target::new(TargetKind::LogDir {
                        file_name: Some("telescope".to_string()),
                    }),
                ])
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_vue::init())
        .plugin(tauri_plugin_cache::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir");
            let _ = std::fs::create_dir_all(&app_dir);
            let initial_state = IntelState::load(&app_dir);
            app.manage(Mutex::new(initial_state));
            app.manage(app_dir);
            #[cfg(any(target_os = "linux", all(debug_assertions, windows)))]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                let _ = app.deep_link().register_all();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::lookup_pilots,
            commands::ensure_sde_index,
            commands::get_sde_status,
            commands::parse_dscan,
            commands::clear_cache,
            commands::check_for_update,
            commands::is_overlay_open,
            commands::open_overlay,
            commands::close_overlay,
            commands::toggle_overlay,
            intel_commands::get_intel_state,
            intel_commands::set_api_base_url,
            intel_commands::set_api_token,
            intel_commands::logout_intel,
            intel_commands::set_active_network_ids,
            intel_commands::fetch_networks,
            intel_commands::create_network,
            intel_commands::delete_network,
            intel_commands::select_network,
            intel_commands::clear_selected_network,
            intel_commands::lookup_intel,
            intel_commands::add_intel_entry,
            intel_commands::update_intel_entry,
            intel_commands::remove_intel_entry,
            intel_commands::add_network_access,
            intel_commands::remove_network_access,
            intel_commands::search_entities,
            intel_commands::share_scan,
            intel_commands::fetch_network_scans,
            intel_commands::fetch_network_scan,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
