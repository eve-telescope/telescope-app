//! Accumulates pilot results streamed in during a scan.

use std::collections::HashMap;

use crate::models::PilotIntel;

/// Insertion-ordered pilots keyed by [`PilotIntel::row_key`]. Upserts are
/// O(1); an upsert for a known key replaces the entry in place, keeping its
/// position. Views sort on their own, so the accumulator only preserves
/// arrival order.
#[derive(Debug, Default, Clone)]
pub struct PilotAccumulator {
    pilots: Vec<PilotIntel>,
    index: HashMap<u64, usize>,
}

impl PilotAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn upsert(&mut self, pilot: PilotIntel) {
        match self.index.get(&pilot.row_key()) {
            Some(&i) => self.pilots[i] = pilot,
            None => {
                self.index.insert(pilot.row_key(), self.pilots.len());
                self.pilots.push(pilot);
            }
        }
    }

    /// Drops pilots not matching the predicate. A new scan keeps the pilots
    /// it shares with the previous one so their rows cross-fade.
    pub fn retain_where(&mut self, mut predicate: impl FnMut(&PilotIntel) -> bool) {
        self.pilots.retain(|p| predicate(p));
        self.reindex();
    }

    pub fn iter(&self) -> std::slice::Iter<'_, PilotIntel> {
        self.pilots.iter()
    }

    pub fn as_slice(&self) -> &[PilotIntel] {
        &self.pilots
    }

    pub fn to_vec(&self) -> Vec<PilotIntel> {
        self.pilots.clone()
    }

    pub fn get(&self, row_key: u64) -> Option<&PilotIntel> {
        self.index.get(&row_key).map(|&i| &self.pilots[i])
    }

    pub fn clear(&mut self) {
        self.pilots.clear();
        self.index.clear();
    }

    pub fn len(&self) -> usize {
        self.pilots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pilots.is_empty()
    }

    fn reindex(&mut self) {
        self.index.clear();
        for (i, p) in self.pilots.iter().enumerate() {
            self.index.insert(p.row_key(), i);
        }
    }
}

impl<'a> IntoIterator for &'a PilotAccumulator {
    type Item = &'a PilotIntel;
    type IntoIter = std::slice::Iter<'a, PilotIntel>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::test_support::{names, pilot, with_threat};

    fn make(id: i64, threat: &str, name: &str) -> PilotIntel {
        with_threat(pilot(id, name), threat)
    }

    #[test]
    fn adds_pilots_and_reports_size() {
        let mut acc = PilotAccumulator::new();
        acc.upsert(make(1, "HIGH", "Pilot 1"));
        acc.upsert(make(2, "LOW", "Pilot 2"));
        assert_eq!(acc.len(), 2);
    }

    #[test]
    fn replaces_entry_for_existing_character_id() {
        let mut acc = PilotAccumulator::new();
        acc.upsert(make(1, "LOW", "Stale"));
        acc.upsert(make(1, "HIGH", "Fresh"));
        assert_eq!(acc.len(), 1);
        assert_eq!(acc.to_vec()[0].character.name, "Fresh");
        assert_eq!(acc.to_vec()[0].threat_level, "HIGH");
    }

    #[test]
    fn retain_where_keeps_only_matching_pilots() {
        let mut acc = PilotAccumulator::new();
        acc.upsert(make(1, "HIGH", "Keep Me"));
        acc.upsert(make(2, "LOW", "Drop Me"));
        acc.upsert(make(3, "EXTREME", "Also Keep"));

        let wanted = ["keep me", "also keep"];
        acc.retain_where(|p| wanted.contains(&p.character.name.to_lowercase().as_str()));

        assert_eq!(acc.len(), 2);
        assert_eq!(names(&acc), vec!["Keep Me", "Also Keep"]);
        assert_eq!(acc.get(3).unwrap().character.name, "Also Keep");
        assert!(acc.get(2).is_none());
    }

    #[test]
    fn returns_pilots_in_insertion_order() {
        let mut acc = PilotAccumulator::new();
        for (id, threat) in [(1, "Unknown"), (2, "LOW"), (3, "EXTREME"), (4, "MODERATE")] {
            acc.upsert(make(id, threat, "p"));
        }
        let threats: Vec<_> = acc.iter().map(|p| p.threat_level.as_str()).collect();
        assert_eq!(threats, vec!["Unknown", "LOW", "EXTREME", "MODERATE"]);
    }

    #[test]
    fn upserting_existing_pilot_keeps_position() {
        let mut acc = PilotAccumulator::new();
        acc.upsert(make(1, "HIGH", "First"));
        acc.upsert(make(2, "EXTREME", "Second"));
        acc.upsert(make(1, "LOW", "First Updated"));
        assert_eq!(names(&acc), vec!["First Updated", "Second"]);
    }

    #[test]
    fn clear_empties_the_accumulator() {
        let mut acc = PilotAccumulator::new();
        acc.upsert(make(1, "HIGH", "p"));
        acc.clear();
        assert_eq!(acc.len(), 0);
        assert!(acc.to_vec().is_empty());
        acc.upsert(make(1, "HIGH", "p"));
        assert_eq!(acc.len(), 1);
    }

    #[test]
    fn unresolved_pilots_are_kept_apart_by_name() {
        let mut acc = PilotAccumulator::new();
        acc.upsert(make(0, "Unknown", "Typo One"));
        acc.upsert(make(0, "Unknown", "Typo Two"));
        acc.upsert(make(0, "Unknown", "typo one"));
        assert_eq!(names(&acc), vec!["typo one", "Typo Two"]);
        assert_ne!(make(0, "Unknown", "Typo One").row_key(), 0);
    }
}
