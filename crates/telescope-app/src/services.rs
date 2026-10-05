use std::sync::Arc;

use gpui_kit::{App, Global};
use telescope_core::cache::Cache;
use telescope_core::config;
use telescope_core::intel_service::IntelService;
use telescope_core::paths::AppPaths;
use telescope_core::sde::SdeService;

/// Long-lived backend services shared by every window.
pub struct Services {
    pub paths: AppPaths,
    pub cache: Cache,
    pub intel: Arc<IntelService>,
    pub sde: Arc<SdeService>,
    pub api_base_url: String,
}

impl Global for Services {}

impl Services {
    pub fn new(paths: AppPaths) -> Self {
        let api_base_url = config::api_base_url();
        Self {
            cache: Cache::open(&paths.cache.join("cache.json")),
            intel: Arc::new(IntelService::new(paths.data.clone(), api_base_url.clone())),
            sde: Arc::new(SdeService::default()),
            paths,
            api_base_url,
        }
    }

    pub fn get(cx: &App) -> &Self {
        cx.global::<Self>()
    }
}
