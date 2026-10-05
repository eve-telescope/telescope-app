pub mod filters;
pub mod intel;
pub mod scan;
pub mod settings;

use gpui_kit::{App, Entity, Global};

use filters::FilterStore;
use intel::IntelStore;
use scan::ScanStore;
use settings::SettingsStore;

/// Handles to the app-wide state entities, shared by every window.
#[derive(Clone)]
pub struct Stores {
    pub settings: Entity<SettingsStore>,
    pub intel: Entity<IntelStore>,
    pub scan: Entity<ScanStore>,
    pub filters: Entity<FilterStore>,
}

impl Global for Stores {}

impl Stores {
    pub fn init(cx: &mut App) {
        let settings = settings::init(cx);
        let intel = intel::init(cx);
        let filters = filters::init(intel.clone(), cx);
        let scan = scan::init(intel.clone(), filters.clone(), cx);
        cx.set_global(Self {
            settings,
            intel,
            scan,
            filters,
        });
    }

    pub fn get(cx: &App) -> Self {
        cx.global::<Self>().clone()
    }
}
