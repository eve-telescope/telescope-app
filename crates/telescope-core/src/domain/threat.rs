//! Threat scoring and pilot-flag detection from zKillboard stats.

use crate::models::{PilotFlags, ZkillStats};

mod ship_groups {
    pub const FORCE_RECON: i64 = 833;
    pub const COMBAT_RECON: i64 = 906;
    pub const BLACK_OPS: i64 = 898;
    pub const COVERT_OPS: i64 = 830;
    pub const STEALTH_BOMBER: i64 = 834;
    pub const BLOCKADE_RUNNER: i64 = 1202;
    pub const EXPEDITION_FRIGATE: i64 = 1283;
    pub const DREADNOUGHT: i64 = 485;
    pub const CARRIER: i64 = 547;
    pub const FORCE_AUXILIARY: i64 = 1538;
    pub const CAPITAL_INDUSTRIAL: i64 = 883;
    pub const SUPERCARRIER: i64 = 659;
    pub const TITAN: i64 = 30;
    pub const LANCER_DREADNOUGHT: i64 = 4594;
    pub const MARAUDER: i64 = 900;
    pub const HEAVY_INTERDICTOR: i64 = 894;
    pub const INTERDICTOR: i64 = 541;
    pub const STRATEGIC_CRUISER: i64 = 963;
    pub const COMMAND_SHIP: i64 = 540;
}

const COVERT_CYNO_GROUPS: &[i64] = &[
    ship_groups::FORCE_RECON,
    ship_groups::BLACK_OPS,
    ship_groups::COVERT_OPS,
    ship_groups::STEALTH_BOMBER,
    ship_groups::BLOCKADE_RUNNER,
    ship_groups::EXPEDITION_FRIGATE,
];

const RECON_GROUPS: &[i64] = &[ship_groups::FORCE_RECON, ship_groups::COMBAT_RECON];

const CAPITAL_GROUPS: &[i64] = &[
    ship_groups::DREADNOUGHT,
    ship_groups::CARRIER,
    ship_groups::FORCE_AUXILIARY,
    ship_groups::CAPITAL_INDUSTRIAL,
    ship_groups::SUPERCARRIER,
    ship_groups::TITAN,
];

const SUPER_GROUPS: &[i64] = &[ship_groups::SUPERCARRIER, ship_groups::TITAN];

pub fn detect_pilot_flags(zkill: &Option<ZkillStats>) -> PilotFlags {
    let mut flags = PilotFlags::default();

    let Some(stats) = zkill else {
        return flags;
    };

    let ships = &stats.top_ships;

    let has_ship_in_group = |groups: &[i64], min_appearances: i64| {
        ships
            .iter()
            .any(|s| groups.contains(&s.group_id) && s.kills + s.losses >= min_appearances)
    };

    flags.is_recon = has_ship_in_group(RECON_GROUPS, 1);
    flags.is_blops = has_ship_in_group(&[ship_groups::BLACK_OPS], 1);
    flags.is_cyno = has_ship_in_group(COVERT_CYNO_GROUPS, 1);
    flags.is_capital = has_ship_in_group(CAPITAL_GROUPS, 1);
    flags.is_super = has_ship_in_group(SUPER_GROUPS, 1);

    if stats.ships_destroyed > 10 {
        let solo_ratio = stats.solo_kills as f64 / stats.ships_destroyed as f64;
        flags.is_solo = solo_ratio > 0.3;
    }

    flags
}

/// One part of the danger score, each normalized to 0..=1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DangerPart {
    pub label: &'static str,
    pub weight: f64,
    pub value: f64,
}

/// A 0..=100 rating of how dangerous a pilot is, led by experience and
/// small-gang lethality, then recent activity, effectiveness, the hulls they
/// fly and the danger of their targets.
#[derive(Debug, Clone, PartialEq)]
pub struct DangerScore {
    pub total: u8,
    pub parts: Vec<DangerPart>,
}

/// How much flying a ship group adds to a pilot's danger, 0..=1.
fn role_danger(group_id: i64) -> f64 {
    match group_id {
        ship_groups::FORCE_RECON
        | ship_groups::COMBAT_RECON
        | ship_groups::MARAUDER
        | ship_groups::BLACK_OPS
        | ship_groups::SUPERCARRIER
        | ship_groups::TITAN => 1.0,
        ship_groups::DREADNOUGHT
        | ship_groups::LANCER_DREADNOUGHT
        | ship_groups::HEAVY_INTERDICTOR
        | ship_groups::STEALTH_BOMBER => 0.7,
        ship_groups::CARRIER
        | ship_groups::FORCE_AUXILIARY
        | ship_groups::INTERDICTOR
        | ship_groups::STRATEGIC_CRUISER
        | ship_groups::COMMAND_SHIP => 0.5,
        _ => 0.0,
    }
}

fn saturate(x: f64, scale: f64) -> f64 {
    1.0 - (-x / scale).exp()
}

/// `None` without any recorded kills or losses.
pub fn danger_score(stats: &ZkillStats) -> Option<DangerScore> {
    let kills = stats.ships_destroyed as f64;
    let losses = stats.ships_lost as f64;
    if kills + losses == 0.0 {
        return None;
    }

    let activity = 0.6 * saturate(stats.recent_kills as f64, 40.0)
        + 0.4 * saturate(stats.active_pvp_kills as f64, 10.0);

    // Smoothed so a handful of kills without losses doesn't read as elite.
    let kill_share = (kills + 5.0) / (kills + losses + 10.0);
    let isk_share = (stats.isk_destroyed + 1e9) / (stats.isk_destroyed + stats.isk_lost + 2e9);
    let effectiveness = (((kill_share + isk_share) / 2.0 - 0.4) / 0.55).clamp(0.0, 1.0);

    let experience = ((kills + 1.0).log10() / 3.5).clamp(0.0, 1.0);

    // Gang size and target danger only mean something with kills behind
    // them; a pilot with no kills has no gang size to speak of.
    let confidence = saturate(kills, 20.0);
    let small_gang = 1.0 / stats.avg_attackers.max(1.0).sqrt();
    let solo = (stats.solo_kills as f64 / kills.max(1.0) * 4.0).min(1.0);
    let gang = small_gang.max(solo) * confidence;

    let targets = (stats.danger_ratio / 100.0).clamp(0.0, 1.0) * confidence;

    let roles = stats
        .top_ships
        .iter()
        .filter(|ship| ship.kills + ship.losses > 0)
        .map(|ship| ship.group_id)
        .chain(stats.lost_groups.iter().map(|group| group.group_id))
        .map(role_danger)
        .fold(0.0, f64::max);

    let parts = vec![
        DangerPart {
            label: "Experience",
            weight: 0.30,
            value: experience,
        },
        DangerPart {
            label: "Small-gang lethality",
            weight: 0.28,
            value: gang,
        },
        DangerPart {
            label: "Recent activity",
            weight: 0.15,
            value: activity,
        },
        DangerPart {
            label: "Effectiveness",
            weight: 0.12,
            value: effectiveness,
        },
        DangerPart {
            label: "Dangerous hulls",
            weight: 0.08,
            value: roles,
        },
        DangerPart {
            label: "Target danger",
            weight: 0.07,
            value: targets,
        },
    ];
    let total = parts.iter().map(|p| p.weight * p.value).sum::<f64>() * 100.0;
    Some(DangerScore {
        total: total.round().clamp(0.0, 100.0) as u8,
        parts,
    })
}

pub fn threat_level_for(score: Option<u8>) -> &'static str {
    match score {
        None => "Unknown",
        Some(s) if s >= 70 => "EXTREME",
        Some(s) if s >= 50 => "HIGH",
        Some(s) if s >= 32 => "MODERATE",
        Some(s) if s >= 15 => "LOW",
        Some(_) => "MINIMAL",
    }
}

pub fn calculate_threat_level(zkill: &Option<ZkillStats>) -> String {
    let score = zkill.as_ref().and_then(danger_score).map(|s| s.total);
    threat_level_for(score).to_string()
}

/// Severity rank for the levels `calculate_threat_level` produces:
/// 0 is most dangerous, unknown strings sort last. Defined next to the
/// level vocabulary so tiers and their order can never drift apart.
pub fn threat_rank(level: &str) -> u8 {
    match level {
        "EXTREME" => 0,
        "HIGH" => 1,
        "MODERATE" => 2,
        "LOW" => 3,
        "MINIMAL" => 4,
        _ => 5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ShipStats;

    #[test]
    fn threat_rank_orders_every_level_calculate_can_produce() {
        let levels = ["EXTREME", "HIGH", "MODERATE", "LOW", "MINIMAL"];
        for pair in levels.windows(2) {
            assert!(threat_rank(pair[0]) < threat_rank(pair[1]));
        }
        // Unknown vocabulary always sorts last.
        assert!(threat_rank("Unknown") > threat_rank("MINIMAL"));
        assert!(threat_rank("") > threat_rank("MINIMAL"));
    }

    fn ship(group_id: i64, kills: i64) -> ShipStats {
        ShipStats {
            ship_type_id: 1,
            ship_name: "Ship".to_string(),
            group_id,
            group_name: "Group".to_string(),
            kills,
            losses: 0,
        }
    }

    fn stats_with_ships(ships: Vec<ShipStats>) -> Option<ZkillStats> {
        Some(ZkillStats {
            top_ships: ships,
            ..ZkillStats::default()
        })
    }

    #[test]
    fn no_stats_yields_default_flags() {
        assert_eq!(detect_pilot_flags(&None), PilotFlags::default());
        assert_eq!(
            detect_pilot_flags(&Some(ZkillStats::default())),
            PilotFlags::default()
        );
    }

    #[test]
    fn recon_groups_set_recon_and_cyno() {
        // Force recon (833) is both a recon group and a covert-cyno group.
        let flags = detect_pilot_flags(&stats_with_ships(vec![ship(833, 1)]));
        assert!(flags.is_recon);
        assert!(flags.is_cyno);
        assert!(!flags.is_blops);
        assert!(!flags.is_capital);

        // Combat recon (906) is recon but not covert-cyno.
        let flags = detect_pilot_flags(&stats_with_ships(vec![ship(906, 1)]));
        assert!(flags.is_recon);
        assert!(!flags.is_cyno);
    }

    #[test]
    fn zero_kill_ships_do_not_set_flags() {
        let flags = detect_pilot_flags(&stats_with_ships(vec![ship(833, 0)]));
        assert_eq!(flags, PilotFlags::default());
    }

    #[test]
    fn blops_sets_blops_and_cyno() {
        let flags = detect_pilot_flags(&stats_with_ships(vec![ship(898, 1)]));
        assert!(flags.is_blops);
        assert!(flags.is_cyno);
    }

    #[test]
    fn supers_are_also_capitals() {
        let flags = detect_pilot_flags(&stats_with_ships(vec![ship(30, 1)])); // Titan
        assert!(flags.is_super);
        assert!(flags.is_capital);

        let flags = detect_pilot_flags(&stats_with_ships(vec![ship(485, 1)])); // Dreadnought
        assert!(flags.is_capital);
        assert!(!flags.is_super);
    }

    #[test]
    fn solo_flag_requires_volume_and_ratio() {
        let solo_stats = |destroyed, solo| {
            Some(ZkillStats {
                ships_destroyed: destroyed,
                solo_kills: solo,
                ..ZkillStats::default()
            })
        };

        // > 30% solo of > 10 kills
        assert!(detect_pilot_flags(&solo_stats(100, 40)).is_solo);
        // Ratio below threshold
        assert!(!detect_pilot_flags(&solo_stats(100, 30)).is_solo);
        // Not enough total kills, even at 100% solo
        assert!(!detect_pilot_flags(&solo_stats(10, 10)).is_solo);
    }

    #[test]
    fn threat_unknown_without_data() {
        assert_eq!(calculate_threat_level(&None), "Unknown");
        assert_eq!(
            calculate_threat_level(&Some(ZkillStats::default())),
            "Unknown"
        );
    }

    #[test]
    fn threat_minimal_for_pure_loss_records() {
        let stats = Some(ZkillStats {
            ships_lost: 50,
            danger_ratio: 100.0,
            ..ZkillStats::default()
        });
        assert_eq!(calculate_threat_level(&stats), "MINIMAL");
    }

    fn veteran() -> ZkillStats {
        ZkillStats {
            ships_destroyed: 3000,
            ships_lost: 150,
            isk_destroyed: 900e9,
            isk_lost: 40e9,
            solo_kills: 400,
            danger_ratio: 90.0,
            avg_attackers: 3.0,
            recent_kills: 120,
            active_pvp_kills: 30,
            ..ZkillStats::default()
        }
    }

    #[test]
    fn active_small_gang_veterans_are_extreme() {
        let score = danger_score(&veteran()).unwrap();
        assert!(score.total >= 70, "{score:?}");
        assert_eq!(calculate_threat_level(&Some(veteran())), "EXTREME");
    }

    #[test]
    fn inactivity_lowers_the_score() {
        let idle = ZkillStats {
            recent_kills: 0,
            active_pvp_kills: 0,
            ..veteran()
        };
        let active = danger_score(&veteran()).unwrap().total;
        assert!(danger_score(&idle).unwrap().total + 10 <= active);
    }

    #[test]
    fn blob_pilots_score_below_small_gang_pilots() {
        let blob = ZkillStats {
            avg_attackers: 60.0,
            solo_kills: 0,
            ..veteran()
        };
        assert!(danger_score(&blob).unwrap().total < danger_score(&veteran()).unwrap().total);
    }

    #[test]
    fn few_kills_without_losses_are_not_elite() {
        let lucky = ZkillStats {
            ships_destroyed: 3,
            avg_attackers: 10.0,
            ..ZkillStats::default()
        };
        assert!(danger_score(&lucky).unwrap().total < 32);
    }

    #[test]
    fn dangerous_hulls_add_to_the_score() {
        let mut marauder = veteran();
        marauder.lost_groups = vec![crate::models::GroupLosses {
            group_id: ship_groups::MARAUDER,
            losses: 1,
        }];
        assert!(danger_score(&marauder).unwrap().total > danger_score(&veteran()).unwrap().total);
    }

    #[test]
    fn score_parts_weigh_to_one() {
        let parts = danger_score(&veteran()).unwrap().parts;
        let weight: f64 = parts.iter().map(|p| p.weight).sum();
        assert!((weight - 1.0).abs() < 1e-9);
    }
}
