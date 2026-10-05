use gpui_kit::{App, AppContext as _, Context, Entity, Subscription};
use telescope_core::models::PilotIntel;
use telescope_core::view::pilot_filters::PilotFilters;
use telescope_core::view::pilot_tags::pilot_tag_strings_with_index;

use crate::state::intel::IntelStore;

/// Filters shared by the main window and the overlay.
pub struct FilterStore {
    filters: PilotFilters,
    intel: Entity<IntelStore>,
    _intel_changes: Subscription,
}

impl FilterStore {
    pub fn get(&self) -> &PilotFilters {
        &self.filters
    }

    pub fn update(&mut self, cx: &mut Context<Self>, edit: impl FnOnce(&mut PilotFilters)) {
        edit(&mut self.filters);
        cx.notify();
    }

    pub fn matches(&self, pilot: &PilotIntel, cx: &App) -> bool {
        let index = self.intel.read(cx).annotation_index();
        self.filters
            .matches(pilot, |p| pilot_tag_strings_with_index(p, index))
    }

    /// Drops selected tags that no pilot in `pilots` carries any more.
    pub fn prune_tags(&mut self, pilots: &[PilotIntel], cx: &mut Context<Self>) {
        let before = self.filters.selected_tags.len();
        let intel = self.intel.read(cx);
        let index = intel.annotation_index();
        self.filters
            .prune_tags(pilots, |p| pilot_tag_strings_with_index(p, index));
        if self.filters.selected_tags.len() != before {
            cx.notify();
        }
    }
}

pub fn init(intel: Entity<IntelStore>, cx: &mut App) -> Entity<FilterStore> {
    cx.new(|cx| FilterStore {
        filters: PilotFilters::default(),
        _intel_changes: cx.observe(&intel, |_, _, cx| cx.notify()),
        intel,
    })
}
