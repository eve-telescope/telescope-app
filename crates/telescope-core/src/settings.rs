//! User settings persisted as JSON in the app data dir. Field names match the
//! store the Vue frontend used, so its settings file is imported on first run.

use std::path::{Path, PathBuf};

use log::warn;
use serde::{Deserialize, Serialize};

const FILE: &str = "settings.json";
const LEGACY_FILE: &str = "tauri-plugin-vue/settings.json";

pub const DEFAULT_SHORTCUT: &str = "CommandOrControl+Shift+V";

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WindowBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub global_shortcut: String,
    pub sort_column: String,
    pub sort_direction: String,
    pub overlay_locked: bool,
    pub main_window: Option<WindowBounds>,
    pub overlay_window: Option<WindowBounds>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            global_shortcut: DEFAULT_SHORTCUT.to_string(),
            sort_column: "threat".to_string(),
            sort_direction: "desc".to_string(),
            overlay_locked: false,
            main_window: None,
            overlay_window: None,
        }
    }
}

impl Settings {
    pub fn path(data_dir: &Path) -> PathBuf {
        data_dir.join(FILE)
    }

    pub fn load(data_dir: &Path) -> Self {
        [data_dir.join(FILE), data_dir.join(LEGACY_FILE)]
            .iter()
            .find_map(|path| {
                let bytes = std::fs::read(path).ok()?;
                serde_json::from_slice(&bytes)
                    .inspect_err(|e| warn!("Ignoring unreadable {}: {}", path.display(), e))
                    .ok()
            })
            .unwrap_or_default()
    }

    pub fn save(&self, data_dir: &Path) {
        match serde_json::to_vec_pretty(self) {
            Ok(bytes) => {
                if let Err(e) = std::fs::write(Self::path(data_dir), bytes) {
                    warn!("Failed to save settings: {}", e);
                }
            }
            Err(e) => warn!("Failed to serialize settings: {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_legacy_vue_store_shape() {
        let json = r#"{"globalShortcut":"Alt+X","autoScanOnShortcut":true,"sortColumn":"kd","sortDirection":"asc","overlayLocked":true}"#;
        let settings: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.global_shortcut, "Alt+X");
        assert_eq!(settings.sort_column, "kd");
        assert_eq!(settings.sort_direction, "asc");
        assert!(settings.overlay_locked);
        assert_eq!(settings.main_window, None);
    }

    #[test]
    fn missing_fields_fall_back_to_defaults() {
        let settings: Settings = serde_json::from_str("{}").unwrap();
        assert_eq!(settings, Settings::default());
    }
}
