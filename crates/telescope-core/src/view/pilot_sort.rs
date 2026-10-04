//! Column sorting for the pilot table and overlay.

use std::borrow::Borrow;
use std::cmp::Ordering;

use super::format::{kd_ratio_value, ppk};
use super::text::compare_base;
use crate::models::{PilotIntel, ZkillStats};

/// `Corporation`/`Alliance` sort by full name (main table headers);
/// `Corp`/`Ally` sort by ticker (compact overlay headers).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SortKey {
    #[default]
    Threat,
    Pilot,
    Tags,
    Corporation,
    Corp,
    Alliance,
    Ally,
    Kd,
    Isk,
    Ppk,
    Cpk,
    Active,
    Danger,
}

impl SortKey {
    pub const ALL: [SortKey; 13] = [
        Self::Threat,
        Self::Pilot,
        Self::Tags,
        Self::Corporation,
        Self::Corp,
        Self::Alliance,
        Self::Ally,
        Self::Kd,
        Self::Isk,
        Self::Ppk,
        Self::Cpk,
        Self::Active,
        Self::Danger,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Threat => "threat",
            Self::Pilot => "pilot",
            Self::Tags => "tags",
            Self::Corporation => "corporation",
            Self::Corp => "corp",
            Self::Alliance => "alliance",
            Self::Ally => "ally",
            Self::Kd => "kd",
            Self::Isk => "isk",
            Self::Ppk => "ppk",
            Self::Cpk => "cpk",
            Self::Active => "active",
            Self::Danger => "danger",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.as_str() == s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SortDirection {
    Asc,
    #[default]
    Desc,
}

impl SortDirection {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "asc" => Some(Self::Asc),
            "desc" => Some(Self::Desc),
            _ => None,
        }
    }

    pub fn toggled(self) -> Self {
        match self {
            Self::Asc => Self::Desc,
            Self::Desc => Self::Asc,
        }
    }
}

/// Current sort column and direction. Defaults to threat, descending.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SortState {
    pub key: SortKey,
    pub direction: SortDirection,
}

impl SortState {
    /// Header click: same column flips direction, a new column starts
    /// descending.
    pub fn toggle(&mut self, key: SortKey) {
        if self.key == key {
            self.direction = self.direction.toggled();
        } else {
            self.key = key;
            self.direction = SortDirection::Desc;
        }
    }
}

/// Higher is more dangerous; unrecognized levels rank lowest.
pub fn threat_rank(level: &str) -> u8 {
    match level.to_ascii_lowercase().as_str() {
        "extreme" => 5,
        "high" => 4,
        "moderate" => 3,
        "low" => 2,
        "minimal" => 1,
        _ => 0,
    }
}

/// Weighted so a single SUPER outranks every other flag combined. Only
/// zKillboard flags count, annotation tags do not affect the order.
pub fn tag_score(pilot: &PilotIntel) -> u32 {
    let f = &pilot.flags;
    [
        (f.is_super, 100),
        (f.is_capital, 50),
        (f.is_blops, 25),
        (f.is_recon, 12),
        (f.is_cyno, 6),
        (f.is_solo, 3),
    ]
    .into_iter()
    .filter(|(set, _)| *set)
    .map(|(_, weight)| weight)
    .sum()
}

pub fn pilot_kd_ratio(pilot: &PilotIntel) -> f64 {
    pilot
        .zkill
        .as_ref()
        .map_or(0.0, |z| kd_ratio_value(z.ships_destroyed, z.ships_lost))
}

pub fn pilot_ppk(pilot: &PilotIntel) -> f64 {
    pilot
        .zkill
        .as_ref()
        .map_or(0.0, |z| ppk(z.points_destroyed, z.ships_destroyed))
}

fn zkill_f64(pilot: &PilotIntel, field: impl Fn(&ZkillStats) -> f64) -> f64 {
    pilot.zkill.as_ref().map_or(0.0, field)
}

fn cmp_f64(a: f64, b: f64) -> Ordering {
    a.partial_cmp(&b).unwrap_or(Ordering::Equal)
}

fn cmp_opt_str(a: &Option<String>, b: &Option<String>) -> Ordering {
    compare_base(a.as_deref().unwrap_or(""), b.as_deref().unwrap_or(""))
}

/// Ascending comparison for the given column.
pub fn compare_pilots(a: &PilotIntel, b: &PilotIntel, key: SortKey) -> Ordering {
    let (ca, cb) = (&a.character, &b.character);
    match key {
        SortKey::Threat | SortKey::Danger => a
            .danger
            .cmp(&b.danger)
            .then_with(|| threat_rank(&a.threat_level).cmp(&threat_rank(&b.threat_level))),
        SortKey::Pilot => compare_base(&ca.name, &cb.name),
        SortKey::Tags => tag_score(a).cmp(&tag_score(b)),
        SortKey::Corporation => cmp_opt_str(&ca.corporation_name, &cb.corporation_name),
        SortKey::Corp => cmp_opt_str(&ca.corporation_ticker, &cb.corporation_ticker),
        SortKey::Alliance => cmp_opt_str(&ca.alliance_name, &cb.alliance_name),
        SortKey::Ally => cmp_opt_str(&ca.alliance_ticker, &cb.alliance_ticker),
        SortKey::Kd => cmp_f64(pilot_kd_ratio(a), pilot_kd_ratio(b)),
        SortKey::Isk => cmp_f64(
            zkill_f64(a, |z| z.isk_destroyed),
            zkill_f64(b, |z| z.isk_destroyed),
        ),
        SortKey::Ppk => cmp_f64(pilot_ppk(a), pilot_ppk(b)),
        SortKey::Cpk => cmp_f64(
            zkill_f64(a, |z| z.avg_attackers),
            zkill_f64(b, |z| z.avg_attackers),
        ),
        SortKey::Active => cmp_f64(
            zkill_f64(a, |z| z.active_pvp_kills as f64),
            zkill_f64(b, |z| z.active_pvp_kills as f64),
        ),
    }
}

/// Stable in-place sort; ties keep arrival order in both directions.
/// Works on owned pilots or on references (e.g. a filtered `Vec<&PilotIntel>`).
pub fn sort_pilots<P: Borrow<PilotIntel>>(
    pilots: &mut [P],
    key: SortKey,
    direction: SortDirection,
) {
    pilots.sort_by(|a, b| {
        let ord = compare_pilots(a.borrow(), b.borrow(), key);
        match direction {
            SortDirection::Asc => ord,
            SortDirection::Desc => ord.reverse(),
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::test_support::{names, pilot, with_flags, with_threat, with_zkill};

    fn named(name: &str) -> PilotIntel {
        pilot(1, name)
    }

    #[test]
    fn ranks_threat_levels() {
        let levels = ["extreme", "high", "moderate", "low", "minimal", "unknown"];
        let ranks: Vec<_> = levels.iter().map(|l| threat_rank(l)).collect();
        assert_eq!(ranks, vec![5, 4, 3, 2, 1, 0]);
    }

    #[test]
    fn threat_rank_is_case_insensitive() {
        assert_eq!(threat_rank("EXTREME"), 5);
        assert_eq!(threat_rank("Unknown"), 0);
    }

    #[test]
    fn unrecognized_threat_ranks_lowest() {
        assert_eq!(threat_rank("bogus"), 0);
        assert_eq!(threat_rank(""), 0);
    }

    #[test]
    fn tag_score_zero_without_flags() {
        assert_eq!(tag_score(&named("p")), 0);
    }

    #[test]
    fn super_outweighs_everything_else() {
        let super_only = tag_score(&with_flags(named("p"), |f| f.is_super = true));
        let all_others = tag_score(&with_flags(named("p"), |f| {
            f.is_capital = true;
            f.is_blops = true;
            f.is_recon = true;
            f.is_cyno = true;
            f.is_solo = true;
        }));
        assert_eq!(super_only, 100);
        assert_eq!(all_others, 96);
    }

    #[test]
    fn tag_score_sums_weights() {
        let p = with_flags(named("p"), |f| {
            f.is_recon = true;
            f.is_solo = true;
        });
        assert_eq!(tag_score(&p), 15);
    }

    #[test]
    fn kd_ratio_of_pilot() {
        assert_eq!(pilot_kd_ratio(&named("p")), 0.0);
        let p = with_zkill(named("p"), |z| {
            z.ships_destroyed = 30;
            z.ships_lost = 10;
        });
        assert_eq!(pilot_kd_ratio(&p), 3.0);
        let p = with_zkill(named("p"), |z| z.ships_destroyed = 42);
        assert_eq!(pilot_kd_ratio(&p), 42.0);
    }

    #[test]
    fn ppk_of_pilot() {
        assert_eq!(pilot_ppk(&named("p")), 0.0);
        assert_eq!(
            pilot_ppk(&with_zkill(named("p"), |z| z.points_destroyed = 100)),
            0.0
        );
        let p = with_zkill(named("p"), |z| {
            z.points_destroyed = 100;
            z.ships_destroyed = 20;
        });
        assert_eq!(pilot_ppk(&p), 5.0);
    }

    #[test]
    fn corporation_by_name_and_corp_by_ticker() {
        let mut a = named("a");
        a.character.corporation_name = Some("Alpha Corp".into());
        a.character.corporation_ticker = Some("ZZZ".into());
        let mut b = named("b");
        b.character.corporation_name = Some("Zulu Corp".into());
        b.character.corporation_ticker = Some("AAA".into());
        assert_eq!(compare_pilots(&a, &b, SortKey::Corporation), Ordering::Less);
        assert_eq!(compare_pilots(&a, &b, SortKey::Corp), Ordering::Greater);
    }

    #[test]
    fn alliance_by_name_and_ally_by_ticker() {
        let mut a = named("a");
        a.character.alliance_name = Some("Alpha Alliance".into());
        a.character.alliance_ticker = Some("ZZZ".into());
        let mut b = named("b");
        b.character.alliance_name = Some("Zulu Alliance".into());
        b.character.alliance_ticker = Some("AAA".into());
        assert_eq!(compare_pilots(&a, &b, SortKey::Alliance), Ordering::Less);
        assert_eq!(compare_pilots(&a, &b, SortKey::Ally), Ordering::Greater);
    }

    #[test]
    fn missing_names_compare_as_empty() {
        let a = named("a");
        let mut b = named("b");
        b.character.corporation_name = Some("Some Corp".into());
        assert_eq!(compare_pilots(&a, &b, SortKey::Corporation), Ordering::Less);
        assert_eq!(
            compare_pilots(&a, &a, SortKey::Corporation),
            Ordering::Equal
        );
    }

    #[test]
    fn sort_key_round_trips_and_rejects_unknown() {
        for key in SortKey::ALL {
            assert_eq!(SortKey::parse(key.as_str()), Some(key));
        }
        assert_eq!(SortKey::parse("nonsense"), None);
        assert_eq!(SortDirection::parse("asc"), Some(SortDirection::Asc));
        assert_eq!(SortDirection::parse("up"), None);
    }

    #[test]
    fn sort_state_toggle() {
        let mut state = SortState::default();
        assert_eq!(
            state,
            SortState {
                key: SortKey::Threat,
                direction: SortDirection::Desc
            }
        );
        state.toggle(SortKey::Threat);
        assert_eq!(state.direction, SortDirection::Asc);
        state.toggle(SortKey::Pilot);
        assert_eq!(
            state,
            SortState {
                key: SortKey::Pilot,
                direction: SortDirection::Desc
            }
        );
    }

    #[test]
    fn sorts_by_threat_descending() {
        let mut pilots = vec![
            with_threat(named("Low"), "low"),
            with_threat(named("Extreme"), "extreme"),
            with_threat(named("Unknown"), "unknown"),
            with_threat(named("High"), "high"),
        ];
        sort_pilots(&mut pilots, SortKey::Threat, SortDirection::Desc);
        assert_eq!(names(&pilots), vec!["Extreme", "High", "Low", "Unknown"]);
    }

    #[test]
    fn ascending_inverts_order() {
        let mut pilots = vec![
            with_threat(named("Extreme"), "extreme"),
            with_threat(named("Low"), "low"),
        ];
        sort_pilots(&mut pilots, SortKey::Threat, SortDirection::Asc);
        assert_eq!(names(&pilots), vec!["Low", "Extreme"]);
    }

    #[test]
    fn sorts_by_pilot_name_case_insensitively() {
        let mut pilots = vec![named("charlie"), named("ALICE"), named("Bob")];
        sort_pilots(&mut pilots, SortKey::Pilot, SortDirection::Asc);
        assert_eq!(names(&pilots), vec!["ALICE", "Bob", "charlie"]);
    }

    #[test]
    fn names_differing_only_by_case_keep_arrival_order() {
        let mut pilots = vec![named("alice"), named("Alice")];
        sort_pilots(&mut pilots, SortKey::Pilot, SortDirection::Asc);
        assert_eq!(names(&pilots), vec!["alice", "Alice"]);
        sort_pilots(&mut pilots, SortKey::Pilot, SortDirection::Desc);
        assert_eq!(names(&pilots), vec!["alice", "Alice"]);
    }

    #[test]
    fn sorts_by_kd_with_lossless_pilots_ranked_by_kills() {
        let mut pilots = vec![
            with_zkill(named("Ratio3"), |z| {
                z.ships_destroyed = 30;
                z.ships_lost = 10;
            }),
            with_zkill(named("NoLosses"), |z| z.ships_destroyed = 50),
            named("NoData"),
        ];
        sort_pilots(&mut pilots, SortKey::Kd, SortDirection::Desc);
        assert_eq!(names(&pilots), vec!["NoLosses", "Ratio3", "NoData"]);
    }

    #[test]
    fn sorts_references_without_touching_source() {
        let source = vec![
            with_threat(named("B"), "low"),
            with_threat(named("A"), "extreme"),
        ];
        let mut refs: Vec<&PilotIntel> = source.iter().collect();
        sort_pilots(&mut refs, SortKey::Threat, SortDirection::Desc);
        assert_eq!(names(refs), vec!["A", "B"]);
        assert_eq!(names(&source), vec!["B", "A"]);
    }

    #[test]
    fn sorts_by_numeric_zkill_columns() {
        let mut pilots = vec![
            with_zkill(named("Small"), |z| {
                z.isk_destroyed = 1.0;
                z.avg_attackers = 9.0;
                z.active_pvp_kills = 1;
                z.danger_ratio = 10.0;
            }),
            with_zkill(named("Big"), |z| {
                z.isk_destroyed = 5.0;
                z.avg_attackers = 2.0;
                z.active_pvp_kills = 7;
                z.danger_ratio = 90.0;
            }),
        ];
        sort_pilots(&mut pilots, SortKey::Isk, SortDirection::Desc);
        assert_eq!(names(&pilots), vec!["Big", "Small"]);
        sort_pilots(&mut pilots, SortKey::Cpk, SortDirection::Desc);
        assert_eq!(names(&pilots), vec!["Small", "Big"]);
        sort_pilots(&mut pilots, SortKey::Active, SortDirection::Desc);
        assert_eq!(names(&pilots), vec!["Big", "Small"]);
    }

    #[test]
    fn threat_and_danger_sort_by_score() {
        let scored = |name: &str, danger: Option<u8>, level: &str| {
            let mut p = named(name);
            p.danger = danger;
            p.threat_level = level.into();
            p
        };
        let mut pilots = vec![
            scored("Mid", Some(55), "HIGH"),
            scored("None", None, "Unknown"),
            scored("Top", Some(81), "EXTREME"),
            scored("Upper", Some(68), "HIGH"),
        ];
        sort_pilots(&mut pilots, SortKey::Threat, SortDirection::Desc);
        assert_eq!(names(&pilots), vec!["Top", "Upper", "Mid", "None"]);
        sort_pilots(&mut pilots, SortKey::Danger, SortDirection::Asc);
        assert_eq!(names(&pilots), vec!["None", "Mid", "Upper", "Top"]);
    }
}
