//! Pilot list filtering shared by the main window and the overlay.
//!
//! Categories AND together; multi-select sets OR within their category.
//! The main window drives the threat filter, tags and corp/alliance name
//! sets; the overlay drives the threat filter, tags and single corp/alliance
//! tickers. Both use the same [`PilotFilters`] and sync via [`FilterSync`].

use std::collections::{BTreeSet, HashSet};

use crate::models::PilotIntel;

/// Corporation name used for pilots with no corporation.
pub const UNKNOWN_CORPORATION: &str = "Unknown";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PilotFilters {
    /// Lowercased threat level ("extreme", "high", ...).
    pub threat: Option<String>,
    /// Flag labels and annotation tags, as produced by the pilot tag helpers.
    pub selected_tags: BTreeSet<String>,
    /// Single-select corporation ticker.
    pub corp_ticker: Option<String>,
    /// Single-select alliance ticker.
    pub alliance_ticker: Option<String>,
    /// Multi-select corporation names; [`UNKNOWN_CORPORATION`] matches
    /// pilots without a corporation.
    pub selected_corps: BTreeSet<String>,
    /// Multi-select alliance names; pilots without an alliance never match.
    pub selected_alliances: BTreeSet<String>,
}

fn toggle_option(slot: &mut Option<String>, value: &str) {
    if slot.as_deref() == Some(value) {
        *slot = None;
    } else {
        *slot = Some(value.to_string());
    }
}

fn toggle_in_set(set: &mut BTreeSet<String>, value: &str) {
    if !set.remove(value) {
        set.insert(value.to_string());
    }
}

impl PilotFilters {
    pub fn has_active_filters(&self) -> bool {
        self.threat.is_some()
            || !self.selected_tags.is_empty()
            || self.corp_ticker.is_some()
            || self.alliance_ticker.is_some()
            || !self.selected_corps.is_empty()
            || !self.selected_alliances.is_empty()
    }

    pub fn toggle_threat(&mut self, level: &str) {
        toggle_option(&mut self.threat, level);
    }

    pub fn toggle_tag(&mut self, tag: &str) {
        toggle_in_set(&mut self.selected_tags, tag);
    }

    pub fn toggle_corp_ticker(&mut self, ticker: &str) {
        toggle_option(&mut self.corp_ticker, ticker);
    }

    pub fn toggle_alliance_ticker(&mut self, ticker: &str) {
        toggle_option(&mut self.alliance_ticker, ticker);
    }

    pub fn toggle_corp(&mut self, name: &str) {
        toggle_in_set(&mut self.selected_corps, name);
    }

    pub fn toggle_alliance(&mut self, name: &str) {
        toggle_in_set(&mut self.selected_alliances, name);
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// `tags_of` is only called when a tag filter is active and every other
    /// category already matched.
    pub fn matches(
        &self,
        pilot: &PilotIntel,
        tags_of: impl FnOnce(&PilotIntel) -> Vec<String>,
    ) -> bool {
        let c = &pilot.character;

        if let Some(threat) = &self.threat
            && !threat.is_empty()
            && pilot.threat_level.to_lowercase() != *threat
        {
            return false;
        }
        if let Some(ticker) = self.corp_ticker.as_deref().filter(|t| !t.is_empty())
            && c.corporation_ticker.as_deref() != Some(ticker)
        {
            return false;
        }
        if let Some(ticker) = self.alliance_ticker.as_deref().filter(|t| !t.is_empty())
            && c.alliance_ticker.as_deref() != Some(ticker)
        {
            return false;
        }
        if !self.selected_corps.is_empty() {
            let name = c
                .corporation_name
                .as_deref()
                .filter(|n| !n.is_empty())
                .unwrap_or(UNKNOWN_CORPORATION);
            if !self.selected_corps.contains(name) {
                return false;
            }
        }
        if !self.selected_alliances.is_empty()
            && !c
                .alliance_name
                .as_deref()
                .is_some_and(|n| self.selected_alliances.contains(n))
        {
            return false;
        }
        if !self.selected_tags.is_empty()
            && !tags_of(pilot)
                .iter()
                .any(|t| self.selected_tags.contains(t))
        {
            return false;
        }
        true
    }

    /// Drops selected tags no pilot carries any more (intel entries changed).
    /// Does nothing while there are no pilots, so a cleared list keeps the
    /// selection for the next scan.
    pub fn prune_tags<'a>(
        &mut self,
        pilots: impl IntoIterator<Item = &'a PilotIntel>,
        mut tags_of: impl FnMut(&PilotIntel) -> Vec<String>,
    ) {
        if self.selected_tags.is_empty() {
            return;
        }
        let mut any = false;
        let mut available = HashSet::new();
        for pilot in pilots {
            any = true;
            available.extend(tags_of(pilot));
        }
        if any {
            self.selected_tags.retain(|t| available.contains(t));
        }
    }

    pub fn sync_payload(&self, side: FilterSide) -> FilterSync {
        let tags = self.selected_tags.iter().cloned().collect();
        match side {
            FilterSide::Main => FilterSync {
                threat: self.threat.clone(),
                selected_tags: tags,
                corp_ticker: None,
                alliance_ticker: None,
                selected_corps: Some(self.selected_corps.iter().cloned().collect()),
                selected_alliances: Some(self.selected_alliances.iter().cloned().collect()),
            },
            FilterSide::Overlay => FilterSync {
                threat: self.threat.clone(),
                selected_tags: tags,
                corp_ticker: self.corp_ticker.clone(),
                alliance_ticker: self.alliance_ticker.clone(),
                selected_corps: None,
                selected_alliances: None,
            },
        }
    }

    /// Applies a payload from the other window. The main window takes the
    /// name sets (missing means empty) and keeps its own tickers; the overlay
    /// takes the tickers (missing means none) and has no name sets.
    pub fn apply_sync(&mut self, side: FilterSide, sync: &FilterSync) {
        self.threat = sync.threat.clone();
        self.selected_tags = sync.selected_tags.iter().cloned().collect();
        match side {
            FilterSide::Main => {
                self.selected_corps = sync.selected_corps.iter().flatten().cloned().collect();
                self.selected_alliances =
                    sync.selected_alliances.iter().flatten().cloned().collect();
            }
            FilterSide::Overlay => {
                self.corp_ticker = sync.corp_ticker.clone();
                self.alliance_ticker = sync.alliance_ticker.clone();
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterSide {
    Main,
    Overlay,
}

/// Cross-window filter payload. Each side fills only the fields it owns.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FilterSync {
    pub threat: Option<String>,
    pub selected_tags: Vec<String>,
    pub corp_ticker: Option<String>,
    pub alliance_ticker: Option<String>,
    pub selected_corps: Option<Vec<String>>,
    pub selected_alliances: Option<Vec<String>>,
}

pub fn has_active_filters(filters: &PilotFilters) -> bool {
    filters.has_active_filters()
}

/// Pilots passing every active filter, in input order.
pub fn filter_pilots<'a>(
    pilots: &'a [PilotIntel],
    filters: &PilotFilters,
    mut tags_of: impl FnMut(&PilotIntel) -> Vec<String>,
) -> Vec<&'a PilotIntel> {
    if !filters.has_active_filters() {
        return pilots.iter().collect();
    }
    pilots
        .iter()
        .filter(|p| filters.matches(p, &mut tags_of))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::IntelEntry;
    use crate::view::annotations::{
        AnnotationIndex, annotations_by_target_key, annotations_from_entries,
    };
    use crate::view::pilot_tags::{pilot_tag_strings, pilot_tag_strings_with_index};
    use crate::view::test_support::{
        affiliated_pilot, flying_recon, names, with_flags, with_threat,
    };

    fn flag_tags(p: &PilotIntel) -> Vec<String> {
        pilot_tag_strings(p, &[])
    }

    fn set(items: &[&str]) -> BTreeSet<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    fn p(id: i64) -> PilotIntel {
        affiliated_pilot(id, &format!("Pilot {id}"))
    }

    #[test]
    fn inactive_for_empty_state() {
        assert!(!PilotFilters::default().has_active_filters());
    }

    #[test]
    fn active_when_any_category_is_set() {
        let cases = [
            PilotFilters {
                threat: Some("high".into()),
                ..Default::default()
            },
            PilotFilters {
                selected_tags: set(&["CYNO"]),
                ..Default::default()
            },
            PilotFilters {
                corp_ticker: Some("TSTC".into()),
                ..Default::default()
            },
            PilotFilters {
                alliance_ticker: Some("TSTA".into()),
                ..Default::default()
            },
            PilotFilters {
                selected_corps: set(&["Test Corp"]),
                ..Default::default()
            },
            PilotFilters {
                selected_alliances: set(&["Test Alliance"]),
                ..Default::default()
            },
        ];
        for f in cases {
            assert!(has_active_filters(&f), "{f:?}");
        }
    }

    #[test]
    fn returns_everything_when_inactive() {
        let pilots = vec![p(1), p(2)];
        assert_eq!(
            filter_pilots(&pilots, &PilotFilters::default(), flag_tags).len(),
            2
        );
    }

    #[test]
    fn filters_by_threat_case_insensitively() {
        let pilots = vec![with_threat(p(1), "Extreme"), with_threat(p(2), "low")];
        let f = PilotFilters {
            threat: Some("extreme".into()),
            ..Default::default()
        };
        let result = filter_pilots(&pilots, &f, flag_tags);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].threat_level, "Extreme");
    }

    #[test]
    fn empty_threat_string_is_ignored() {
        let pilots = vec![p(1)];
        let f = PilotFilters {
            threat: Some(String::new()),
            ..Default::default()
        };
        assert_eq!(filter_pilots(&pilots, &f, flag_tags).len(), 1);
    }

    #[test]
    fn filters_by_flag_tag() {
        let pilots = vec![with_flags(p(1), |f| f.is_cyno = true), p(2)];
        let f = PilotFilters {
            selected_tags: set(&["CYNO"]),
            ..Default::default()
        };
        let result = filter_pilots(&pilots, &f, flag_tags);
        assert_eq!(result.len(), 1);
        assert!(result[0].flags.is_cyno);
    }

    #[test]
    fn ors_tags_within_category() {
        let pilots = vec![
            with_flags(p(1), |f| f.is_cyno = true),
            flying_recon(p(2)),
            p(3),
        ];
        let f = PilotFilters {
            selected_tags: set(&["CYNO", "RECON"]),
            ..Default::default()
        };
        assert_eq!(filter_pilots(&pilots, &f, flag_tags).len(), 2);
    }

    #[test]
    fn supports_injected_tag_resolver() {
        let pilots = vec![affiliated_pilot(1, "Hostile"), p(2)];
        let f = PilotFilters {
            selected_tags: set(&["HOSTILE"]),
            ..Default::default()
        };
        let result = filter_pilots(&pilots, &f, |p| {
            if p.character.id == 1 {
                vec!["HOSTILE".into()]
            } else {
                vec![]
            }
        });
        assert_eq!(names(result), vec!["Hostile"]);
    }

    #[test]
    fn filters_by_annotation_tags_from_index() {
        let pilots = vec![p(1), p(2)];
        let index: AnnotationIndex =
            annotations_by_target_key(annotations_from_entries(&[IntelEntry {
                id: 1,
                intel_network_id: 1,
                network_name: "Net".into(),
                entity_type: "character".into(),
                entity_id: 1,
                entity_name: "Pilot 1".into(),
                color: None,
                label: Some("HOSTILE".into()),
                notes: None,
            }]));
        let mut f = PilotFilters::default();
        f.toggle_tag("HOSTILE");
        let result = filter_pilots(&pilots, &f, |p| pilot_tag_strings_with_index(p, &index));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].character.id, 1);
    }

    #[test]
    fn filters_by_single_select_tickers() {
        let mut other = p(2);
        other.character.corporation_ticker = Some("OTHR".into());
        other.character.alliance_ticker = Some("OTHA".into());
        let pilots = vec![p(1), other];

        let f = PilotFilters {
            corp_ticker: Some("OTHR".into()),
            ..Default::default()
        };
        assert_eq!(
            names(filter_pilots(&pilots, &f, flag_tags)),
            vec!["Pilot 2"]
        );
        let f = PilotFilters {
            alliance_ticker: Some("OTHA".into()),
            ..Default::default()
        };
        assert_eq!(
            names(filter_pilots(&pilots, &f, flag_tags)),
            vec!["Pilot 2"]
        );
    }

    #[test]
    fn missing_corp_matches_unknown() {
        let mut no_corp = p(2);
        no_corp.character.corporation_name = None;
        let pilots = vec![p(1), no_corp];

        let f = PilotFilters {
            selected_corps: set(&["Unknown"]),
            ..Default::default()
        };
        assert_eq!(
            names(filter_pilots(&pilots, &f, flag_tags)),
            vec!["Pilot 2"]
        );
        let f = PilotFilters {
            selected_corps: set(&["Test Corp"]),
            ..Default::default()
        };
        assert_eq!(filter_pilots(&pilots, &f, flag_tags).len(), 1);
    }

    #[test]
    fn missing_alliance_never_matches() {
        let mut no_alliance = p(2);
        no_alliance.character.alliance_name = None;
        let pilots = vec![p(1), no_alliance];
        let f = PilotFilters {
            selected_alliances: set(&["Test Alliance"]),
            ..Default::default()
        };
        assert_eq!(
            names(filter_pilots(&pilots, &f, flag_tags)),
            vec!["Pilot 1"]
        );
    }

    #[test]
    fn ands_across_categories() {
        let pilots = vec![
            with_threat(
                with_flags(affiliated_pilot(1, "match"), |f| f.is_cyno = true),
                "high",
            ),
            with_threat(
                with_flags(affiliated_pilot(2, "wrong threat"), |f| f.is_cyno = true),
                "low",
            ),
            with_threat(affiliated_pilot(3, "wrong tag"), "high"),
        ];
        let f = PilotFilters {
            threat: Some("high".into()),
            selected_tags: set(&["CYNO"]),
            ..Default::default()
        };
        assert_eq!(names(filter_pilots(&pilots, &f, flag_tags)), vec!["match"]);
    }

    #[test]
    fn toggles_and_clear() {
        let pilots = vec![with_threat(p(1), "extreme"), p(2)];
        let mut f = PilotFilters::default();

        f.toggle_threat("extreme");
        assert_eq!(filter_pilots(&pilots, &f, flag_tags).len(), 1);
        f.toggle_threat("extreme");
        assert!(f.threat.is_none());

        f.toggle_threat("extreme");
        f.toggle_tag("CYNO");
        assert!(filter_pilots(&pilots, &f, flag_tags).is_empty());

        f.clear();
        assert!(!f.has_active_filters());
        assert_eq!(filter_pilots(&pilots, &f, flag_tags).len(), 2);
    }

    #[test]
    fn toggle_corp_and_alliance_by_name() {
        let mut a = p(1);
        a.character.corporation_name = Some("Corp A".into());
        a.character.alliance_name = Some("Alliance A".into());
        let mut b = p(2);
        b.character.corporation_name = Some("Corp B".into());
        b.character.alliance_name = Some("Alliance B".into());
        let pilots = vec![a, b];

        let mut f = PilotFilters::default();
        f.toggle_corp("Corp A");
        assert_eq!(
            names(filter_pilots(&pilots, &f, flag_tags)),
            vec!["Pilot 1"]
        );
        f.toggle_corp("Corp B");
        assert_eq!(filter_pilots(&pilots, &f, flag_tags).len(), 2);
        f.toggle_corp("Corp B");
        f.toggle_corp("Corp A");
        f.toggle_alliance("Alliance B");
        assert_eq!(
            names(filter_pilots(&pilots, &f, flag_tags)),
            vec!["Pilot 2"]
        );
    }

    #[test]
    fn prunes_tags_no_pilot_carries() {
        let pilots = vec![with_flags(p(1), |f| f.is_cyno = true)];
        let mut f = PilotFilters {
            selected_tags: set(&["CYNO", "HOSTILE"]),
            ..Default::default()
        };
        f.prune_tags(&pilots, flag_tags);
        assert_eq!(f.selected_tags, set(&["CYNO"]));

        f.prune_tags(&[], flag_tags);
        assert_eq!(f.selected_tags, set(&["CYNO"]));
    }

    #[test]
    fn main_and_overlay_sync() {
        let mut main = PilotFilters {
            threat: Some("high".into()),
            selected_tags: set(&["CYNO"]),
            corp_ticker: Some("MAIN".into()),
            selected_corps: set(&["Test Corp"]),
            ..Default::default()
        };
        let mut overlay = PilotFilters {
            corp_ticker: Some("OLD".into()),
            ..Default::default()
        };

        overlay.apply_sync(FilterSide::Overlay, &main.sync_payload(FilterSide::Main));
        assert_eq!(overlay.threat.as_deref(), Some("high"));
        assert_eq!(overlay.selected_tags, set(&["CYNO"]));
        assert_eq!(overlay.corp_ticker, None);
        assert!(overlay.selected_corps.is_empty());

        overlay.toggle_alliance_ticker("TSTA");
        overlay.toggle_tag("RECON");
        main.apply_sync(FilterSide::Main, &overlay.sync_payload(FilterSide::Overlay));
        assert_eq!(main.selected_tags, set(&["CYNO", "RECON"]));
        assert!(main.selected_corps.is_empty());
        assert_eq!(main.corp_ticker.as_deref(), Some("MAIN"));
        assert_eq!(main.alliance_ticker, None);
    }
}
