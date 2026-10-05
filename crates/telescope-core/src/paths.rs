//! Platform directories. They use the same identifier the Tauri build used,
//! so an existing intel_state.json and SDE index are picked up.

use std::path::PathBuf;

use crate::config::IDENTIFIER;

pub struct AppPaths {
    pub data: PathBuf,
    pub cache: PathBuf,
    pub logs: PathBuf,
}

impl AppPaths {
    pub fn resolve() -> Self {
        let data = dirs::data_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join(IDENTIFIER);
        let cache = dirs::cache_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join(IDENTIFIER);
        #[cfg(target_os = "macos")]
        let logs = dirs::home_dir()
            .map(|home| home.join("Library/Logs").join(IDENTIFIER))
            .unwrap_or_else(|| data.join("logs"));
        #[cfg(not(target_os = "macos"))]
        let logs = data.join("logs");

        for dir in [&data, &cache, &logs] {
            let _ = std::fs::create_dir_all(dir);
        }
        Self { data, cache, logs }
    }
}
