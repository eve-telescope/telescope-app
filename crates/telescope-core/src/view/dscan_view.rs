//! D-scan summary helpers: grouping by type and class, bar widths and the
//! ship-class icon.

use std::collections::HashMap;

use super::text::compare_locale;
use crate::models::DscanEntry;

pub const UNKNOWN_CLASS: &str = "Unknown class";

/// Lucide icon shown next to a ship class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClassIcon {
    HeartPulse,
    CircleDot,
    Zap,
    Crosshair,
    Eye,
    Bomb,
    Satellite,
    Anchor,
    Truck,
    Pickaxe,
    Swords,
    ShieldHalf,
    Shield,
    Rocket,
    Ship,
}

impl ClassIcon {
    pub fn lucide_name(self) -> &'static str {
        match self {
            Self::HeartPulse => "heart-pulse",
            Self::CircleDot => "circle-dot",
            Self::Zap => "zap",
            Self::Crosshair => "crosshair",
            Self::Eye => "eye",
            Self::Bomb => "bomb",
            Self::Satellite => "satellite",
            Self::Anchor => "anchor",
            Self::Truck => "truck",
            Self::Pickaxe => "pickaxe",
            Self::Swords => "swords",
            Self::ShieldHalf => "shield-half",
            Self::Shield => "shield",
            Self::Rocket => "rocket",
            Self::Ship => "ship",
        }
    }
}

/// Checked in order, so specific classes come before broader ones
/// ("battlecruiser" before "cruiser", "capital" before "industrial").
pub const CLASS_ICON_RULES: [(&str, ClassIcon); 29] = [
    ("logistic", ClassIcon::HeartPulse),
    ("force auxiliary", ClassIcon::HeartPulse),
    ("capsule", ClassIcon::CircleDot),
    ("interceptor", ClassIcon::Zap),
    ("interdictor", ClassIcon::Crosshair),
    ("covert", ClassIcon::Eye),
    ("recon", ClassIcon::Eye),
    ("stealth", ClassIcon::Bomb),
    ("bomber", ClassIcon::Bomb),
    ("electronic", ClassIcon::Satellite),
    ("titan", ClassIcon::Anchor),
    ("supercarrier", ClassIcon::Anchor),
    ("carrier", ClassIcon::Anchor),
    ("dreadnought", ClassIcon::Anchor),
    ("capital", ClassIcon::Anchor),
    ("freighter", ClassIcon::Truck),
    ("industrial", ClassIcon::Truck),
    ("transport", ClassIcon::Truck),
    ("hauler", ClassIcon::Truck),
    ("mining", ClassIcon::Pickaxe),
    ("barge", ClassIcon::Pickaxe),
    ("exhumer", ClassIcon::Pickaxe),
    ("command", ClassIcon::Swords),
    ("destroyer", ClassIcon::Swords),
    ("battlecruiser", ClassIcon::ShieldHalf),
    ("marauder", ClassIcon::Shield),
    ("battleship", ClassIcon::Shield),
    ("frigate", ClassIcon::Rocket),
    ("shuttle", ClassIcon::Rocket),
];

pub fn class_icon(class_name: &str) -> ClassIcon {
    let lower = class_name.to_lowercase();
    CLASS_ICON_RULES
        .iter()
        .find(|(keyword, _)| lower.contains(keyword))
        .map_or(ClassIcon::Ship, |&(_, icon)| icon)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeBucket {
    pub type_id: Option<i64>,
    pub type_name: String,
    pub subtitle: String,
    pub count: usize,
}

/// Groups entries by type name. Type id and subtitle come from the first
/// entry of each type. Sorted by count descending, then name.
pub fn bucket_by_type(
    entries: &[DscanEntry],
    subtitle_of: impl Fn(&DscanEntry) -> String,
) -> Vec<TypeBucket> {
    let mut buckets: Vec<TypeBucket> = Vec::new();
    let mut index: HashMap<&str, usize> = HashMap::new();

    for entry in entries {
        match index.get(entry.type_name.as_str()) {
            Some(&i) => buckets[i].count += 1,
            None => {
                index.insert(&entry.type_name, buckets.len());
                buckets.push(TypeBucket {
                    type_id: entry.type_id,
                    type_name: entry.type_name.clone(),
                    subtitle: subtitle_of(entry),
                    count: 1,
                });
            }
        }
    }

    buckets.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then_with(|| compare_locale(&a.type_name, &b.type_name))
    });
    buckets
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassCount {
    pub name: String,
    pub count: usize,
}

/// Entries per ship class (group name). Sorted by count descending, then
/// name.
pub fn count_by_class(entries: &[DscanEntry]) -> Vec<ClassCount> {
    let mut counts: Vec<ClassCount> = Vec::new();
    let mut index: HashMap<&str, usize> = HashMap::new();

    for entry in entries {
        let name = entry.group_name.as_deref().unwrap_or(UNKNOWN_CLASS);
        match index.get(name) {
            Some(&i) => counts[i].count += 1,
            None => {
                index.insert(name, counts.len());
                counts.push(ClassCount {
                    name: name.to_string(),
                    count: 1,
                });
            }
        }
    }

    counts.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then_with(|| compare_locale(&a.name, &b.name))
    });
    counts
}

/// Bar width in percent (0-100), never below `min_percent` so tiny counts
/// stay visible. Zero when `max` is zero.
pub fn bar_width(count: usize, max: usize, min_percent: f64) -> f64 {
    if max == 0 {
        return 0.0;
    }
    min_percent.max(count as f64 / max as f64 * 100.0)
}

/// Ship hull classes with an overview bracket icon in the game client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Hull {
    Frigate,
    MiningFrigate,
    Destroyer,
    Cruiser,
    Battlecruiser,
    Battleship,
    Carrier,
    Dreadnought,
    ForceAuxiliary,
    Supercarrier,
    Titan,
    Freighter,
    Industrial,
    IndustrialCommand,
    MiningBarge,
    Shuttle,
    Capsule,
    Rookie,
}

impl Hull {
    pub const ALL: [Hull; 18] = [
        Hull::Frigate,
        Hull::MiningFrigate,
        Hull::Destroyer,
        Hull::Cruiser,
        Hull::Battlecruiser,
        Hull::Battleship,
        Hull::Carrier,
        Hull::Dreadnought,
        Hull::ForceAuxiliary,
        Hull::Supercarrier,
        Hull::Titan,
        Hull::Freighter,
        Hull::Industrial,
        Hull::IndustrialCommand,
        Hull::MiningBarge,
        Hull::Shuttle,
        Hull::Capsule,
        Hull::Rookie,
    ];

    /// The bracket's file stem, as in `res:/ui/texture/shared/brackets/{stem}_32.png`.
    pub fn bracket(self) -> &'static str {
        match self {
            Hull::Frigate => "frigate",
            Hull::MiningFrigate => "miningfrigate",
            Hull::Destroyer => "destroyer",
            Hull::Cruiser => "cruiser",
            Hull::Battlecruiser => "battlecruiser",
            Hull::Battleship => "battleship",
            Hull::Carrier => "carrier",
            Hull::Dreadnought => "dreadnought",
            Hull::ForceAuxiliary => "forceauxiliary",
            Hull::Supercarrier => "supercarrier",
            Hull::Titan => "titan",
            Hull::Freighter => "freighter",
            Hull::Industrial => "industrial",
            Hull::IndustrialCommand => "industrialcommand",
            Hull::MiningBarge => "miningbarge",
            Hull::Shuttle => "shuttle",
            Hull::Capsule => "capsule",
            Hull::Rookie => "rookie",
        }
    }

    /// The hull for an SDE ship group name, `None` for groups the client
    /// draws with a non-class bracket.
    pub fn for_group(group_name: &str) -> Option<Hull> {
        Some(match group_name {
            "Frigate"
            | "Assault Frigate"
            | "Interceptor"
            | "Covert Ops"
            | "Stealth Bomber"
            | "Electronic Attack Ship"
            | "Logistics Frigate"
            | "Prototype Exploration Ship" => Hull::Frigate,
            "Expedition Frigate" => Hull::MiningFrigate,
            "Destroyer" | "Interdictor" | "Tactical Destroyer" | "Command Destroyer" => {
                Hull::Destroyer
            }
            "Cruiser"
            | "Heavy Assault Cruiser"
            | "Heavy Interdiction Cruiser"
            | "Logistics"
            | "Combat Recon Ship"
            | "Force Recon Ship"
            | "Strategic Cruiser"
            | "Flag Cruiser"
            | "Special Edition Yachts" => Hull::Cruiser,
            "Combat Battlecruiser" | "Attack Battlecruiser" | "Command Ship" => Hull::Battlecruiser,
            "Battleship" | "Black Ops" | "Marauder" => Hull::Battleship,
            "Carrier" | "Command Carrier" => Hull::Carrier,
            "Dreadnought" | "Lancer Dreadnought" => Hull::Dreadnought,
            "Force Auxiliary" => Hull::ForceAuxiliary,
            "Supercarrier" => Hull::Supercarrier,
            "Titan" => Hull::Titan,
            "Freighter" | "Jump Freighter" => Hull::Freighter,
            "Hauler" | "Blockade Runner" | "Deep Space Transport" => Hull::Industrial,
            "Industrial Command Ship" | "Capital Industrial Ship" => Hull::IndustrialCommand,
            "Mining Barge" | "Exhumer" => Hull::MiningBarge,
            "Shuttle" => Hull::Shuttle,
            "Capsule" => Hull::Capsule,
            "Corvette" | "Citizen Ships" => Hull::Rookie,
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(type_name: &str, group_name: Option<&str>) -> DscanEntry {
        DscanEntry {
            type_id: Some(1),
            name: "Some Pilot".into(),
            type_name: type_name.into(),
            distance: None,
            group_name: group_name.map(str::to_string),
            category_name: Some("Ship".into()),
            is_ship: true,
        }
    }

    fn sabre() -> DscanEntry {
        entry("Sabre", Some("Interdictor"))
    }

    #[test]
    fn buckets_count_per_type() {
        let entries = [sabre(), sabre(), entry("Drake", Some("Battlecruiser"))];
        let buckets = bucket_by_type(&entries, |e| {
            e.group_name.clone().unwrap_or("Unknown".into())
        });
        assert_eq!(buckets.len(), 2);
        assert_eq!(
            (buckets[0].type_name.as_str(), buckets[0].count),
            ("Sabre", 2)
        );
        assert_eq!(
            (buckets[1].type_name.as_str(), buckets[1].count),
            ("Drake", 1)
        );
        assert_eq!(buckets[1].subtitle, "Battlecruiser");
    }

    #[test]
    fn buckets_sort_by_count_then_name() {
        let entries = [
            entry("Zealot", None),
            entry("Atron", None),
            entry("Drake", None),
            entry("Drake", None),
        ];
        let names: Vec<_> = bucket_by_type(&entries, |_| String::new())
            .into_iter()
            .map(|b| b.type_name)
            .collect();
        assert_eq!(names, vec!["Drake", "Atron", "Zealot"]);
    }

    #[test]
    fn buckets_use_first_entry_for_subtitle_and_type_id() {
        let mut a = sabre();
        a.type_id = Some(22456);
        let mut b = sabre();
        b.type_id = Some(99999);
        let buckets = bucket_by_type(&[a, b], |e| format!("id:{}", e.type_id.unwrap()));
        assert_eq!(buckets[0].type_id, Some(22456));
        assert_eq!(buckets[0].subtitle, "id:22456");
    }

    #[test]
    fn buckets_empty_for_no_entries() {
        assert!(bucket_by_type(&[], |_| String::new()).is_empty());
    }

    #[test]
    fn counts_by_class_with_unknown_fallback() {
        let entries = [sabre(), sabre(), entry("Sabre", None)];
        assert_eq!(
            count_by_class(&entries),
            vec![
                ClassCount {
                    name: "Interdictor".into(),
                    count: 2
                },
                ClassCount {
                    name: "Unknown class".into(),
                    count: 1
                },
            ]
        );
    }

    #[test]
    fn class_count_ties_break_by_name() {
        let entries = [entry("x", Some("Frigate")), entry("y", Some("Battleship"))];
        let names: Vec<_> = count_by_class(&entries)
            .into_iter()
            .map(|c| c.name)
            .collect();
        assert_eq!(names, vec!["Battleship", "Frigate"]);
    }

    #[test]
    fn battlecruiser_before_battleship() {
        assert_eq!(class_icon("Combat Battlecruiser"), ClassIcon::ShieldHalf);
        assert_eq!(class_icon("Attack Battlecruiser"), ClassIcon::ShieldHalf);
    }

    #[test]
    fn covert_and_recon_share_icon() {
        assert_eq!(class_icon("Combat Recon Ship"), ClassIcon::Eye);
        assert_eq!(class_icon("Covert Ops"), ClassIcon::Eye);
    }

    #[test]
    fn capital_before_industrial() {
        assert_eq!(class_icon("Capital Industrial Ship"), ClassIcon::Anchor);
        assert_eq!(class_icon("Industrial Command Ship"), ClassIcon::Truck);
    }

    #[test]
    fn logistics_icon() {
        assert_eq!(class_icon("Logistics"), ClassIcon::HeartPulse);
        assert_eq!(class_icon("Logistics Frigate"), ClassIcon::HeartPulse);
    }

    #[test]
    fn class_icon_is_case_insensitive() {
        assert_eq!(class_icon("TITAN"), ClassIcon::Anchor);
    }

    #[test]
    fn falls_back_to_ship_icon() {
        assert_eq!(class_icon("Heavy Assault Cruiser"), ClassIcon::Ship);
        assert_eq!(class_icon(""), ClassIcon::Ship);
        assert_eq!(ClassIcon::Ship.lucide_name(), "ship");
    }

    #[test]
    fn bar_width_zero_max() {
        assert_eq!(bar_width(5, 0, 6.0), 0.0);
    }

    #[test]
    fn bar_width_scales_to_max() {
        assert_eq!(bar_width(50, 100, 6.0), 50.0);
        assert_eq!(bar_width(100, 100, 6.0), 100.0);
    }

    #[test]
    fn bar_width_clamps_to_minimum() {
        assert_eq!(bar_width(1, 1000, 6.0), 6.0);
        assert_eq!(bar_width(1, 1000, 4.0), 4.0);
    }

    #[test]
    fn hull_maps_ship_groups_to_brackets() {
        assert_eq!(Hull::for_group("Interceptor"), Some(Hull::Frigate));
        assert_eq!(Hull::for_group("Interdictor"), Some(Hull::Destroyer));
        assert_eq!(Hull::for_group("Logistics"), Some(Hull::Cruiser));
        assert_eq!(Hull::for_group("Command Ship"), Some(Hull::Battlecruiser));
        assert_eq!(Hull::for_group("Black Ops"), Some(Hull::Battleship));
        assert_eq!(Hull::for_group("Jump Freighter"), Some(Hull::Freighter));
        assert_eq!(Hull::for_group("Exhumer"), Some(Hull::MiningBarge));
        assert_eq!(Hull::for_group("Expedition Command Ship"), None);
        assert_eq!(Hull::for_group("Unknown class"), None);
    }
}
