//! Pure d-scan classification: an in-memory type index and the parser that
//! turns raw d-scan paste text into classified rows. All I/O (downloading,
//! building and caching the index) lives in `crate::sde`.

use std::collections::HashMap;

use crate::models::{DscanEntry, DscanParseResult, ScanTypeIndexEntry};

const SHIP_CATEGORY_ID: i64 = 6;

#[derive(Debug, Default)]
pub struct SdeIndex {
    by_type_id: HashMap<i64, ScanTypeIndexEntry>,
    name_to_type_id: HashMap<String, i64>,
}

impl SdeIndex {
    pub fn from_entries(entries: Vec<ScanTypeIndexEntry>) -> Self {
        let mut by_type_id = HashMap::with_capacity(entries.len());
        let mut name_to_type_id = HashMap::with_capacity(entries.len());

        for entry in entries {
            name_to_type_id.insert(normalize_name(&entry.type_name), entry.type_id);
            by_type_id.insert(entry.type_id, entry);
        }

        SdeIndex {
            by_type_id,
            name_to_type_id,
        }
    }

    /// Classify a scan row by type ID, falling back to a name lookup.
    fn classify(&self, type_id: Option<i64>, type_name: &str) -> Option<&ScanTypeIndexEntry> {
        type_id.and_then(|id| self.by_type_id.get(&id)).or_else(|| {
            self.name_to_type_id
                .get(&normalize_name(type_name))
                .and_then(|id| self.by_type_id.get(id))
        })
    }
}

/// Parse raw d-scan text against an in-memory index. Pure function.
pub fn parse_dscan_text(index: &SdeIndex, text: &str) -> DscanParseResult {
    let mut entries = Vec::new();
    let mut ship_count = 0;

    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        let columns: Vec<&str> = line.split('\t').map(str::trim).collect();
        if columns.len() < 3 {
            continue;
        }

        let type_id = columns.first().and_then(|value| value.parse::<i64>().ok());
        let name = columns.get(1).unwrap_or(&"").to_string();
        let type_name = columns.get(2).unwrap_or(&"").to_string();
        let distance = columns
            .get(3)
            .map(|value| value.to_string())
            .filter(|value| !value.is_empty() && value != "-");

        let classification = index.classify(type_id, &type_name);

        let is_ship = classification
            .map(|entry| entry.category_id == SHIP_CATEGORY_ID)
            .unwrap_or(false);

        if is_ship {
            ship_count += 1;
        }

        entries.push(DscanEntry {
            type_id,
            name,
            type_name,
            distance,
            group_name: classification.map(|entry| entry.group_name.clone()),
            category_name: classification.map(|entry| entry.category_name.clone()),
            is_ship,
        });
    }

    DscanParseResult {
        total_rows: entries.len(),
        ship_count,
        entries,
    }
}

fn normalize_name(name: &str) -> String {
    name.trim().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(
        type_id: i64,
        type_name: &str,
        group_name: &str,
        category_id: i64,
        category_name: &str,
    ) -> ScanTypeIndexEntry {
        ScanTypeIndexEntry {
            type_id,
            type_name: type_name.to_string(),
            group_id: 1,
            group_name: group_name.to_string(),
            category_id,
            category_name: category_name.to_string(),
        }
    }

    fn test_index() -> SdeIndex {
        SdeIndex::from_entries(vec![
            entry(587, "Rifter", "Frigate", 6, "Ship"),
            entry(35832, "Astrahus", "Citadel", 65, "Structure"),
        ])
    }

    #[test]
    fn parse_dscan_classifies_ships_by_type_id() {
        let result = parse_dscan_text(&test_index(), "587\tSome Pilot's Rifter\tRifter\t2,3 km");
        assert_eq!(result.total_rows, 1);
        assert_eq!(result.ship_count, 1);
        let row = &result.entries[0];
        assert_eq!(row.type_id, Some(587));
        assert_eq!(row.group_name.as_deref(), Some("Frigate"));
        assert_eq!(row.category_name.as_deref(), Some("Ship"));
        assert!(row.is_ship);
        assert_eq!(row.distance.as_deref(), Some("2,3 km"));
    }

    #[test]
    fn parse_dscan_falls_back_to_name_lookup() {
        // Unknown type ID, but the type name matches (case-insensitively).
        let result = parse_dscan_text(&test_index(), "999999\tUnknown\trifter\t-");
        assert_eq!(result.ship_count, 1);
        assert!(result.entries[0].is_ship);
    }

    #[test]
    fn parse_dscan_dash_distance_is_none() {
        let result = parse_dscan_text(&test_index(), "587\tShip\tRifter\t-");
        assert_eq!(result.entries[0].distance, None);
    }

    #[test]
    fn parse_dscan_skips_short_and_empty_lines() {
        let text = "587\tShip\tRifter\t1 km\n\nnot\ttabs\n just text\n";
        let result = parse_dscan_text(&test_index(), text);
        // "not\ttabs" has 2 columns, "just text" has 1 — both skipped.
        assert_eq!(result.total_rows, 1);
    }

    #[test]
    fn parse_dscan_structures_are_not_ships() {
        let result = parse_dscan_text(&test_index(), "35832\tFortizar Home\tAstrahus\t10 km");
        assert_eq!(result.total_rows, 1);
        assert_eq!(result.ship_count, 0);
        assert_eq!(
            result.entries[0].category_name.as_deref(),
            Some("Structure")
        );
    }

    #[test]
    fn parse_dscan_unknown_type_has_no_classification() {
        let result = parse_dscan_text(&test_index(), "111\tThing\tMystery Object\t5 km");
        let row = &result.entries[0];
        assert_eq!(row.group_name, None);
        assert_eq!(row.category_name, None);
        assert!(!row.is_ship);
    }

    #[test]
    fn normalize_name_trims_and_lowercases() {
        assert_eq!(normalize_name("  Rifter  "), "rifter");
        assert_eq!(normalize_name("ASTRAHUS"), "astrahus");
    }
}
