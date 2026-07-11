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

    let has_ship_in_group = |groups: &[i64], min_kills: i64| {
        ships
            .iter()
            .any(|s| groups.contains(&s.group_id) && s.kills >= min_kills)
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

pub fn calculate_threat_level(zkill: &Option<ZkillStats>) -> String {
    match zkill {
        None => "Unknown".to_string(),
        Some(stats) => {
            if stats.ships_destroyed == 0 && stats.ships_lost == 0 {
                return "Unknown".to_string();
            }

            let mut score = 0.0;

            score += (stats.ships_destroyed as f64).log10().max(0.0) * 10.0;
            score += stats.solo_kills as f64 * 0.5;
            score += stats.danger_ratio * 0.3;
            score -= stats.gang_ratio * 0.1;

            let kd_ratio = if stats.ships_lost > 0 {
                stats.ships_destroyed as f64 / stats.ships_lost as f64
            } else {
                stats.ships_destroyed as f64
            };
            score += kd_ratio.min(10.0) * 5.0;

            if stats.active_pvp_kills > 50 {
                score += 20.0;
            } else if stats.active_pvp_kills > 20 {
                score += 10.0;
            }

            match score {
                s if s >= 80.0 => "EXTREME",
                s if s >= 60.0 => "HIGH",
                s if s >= 40.0 => "MODERATE",
                s if s >= 20.0 => "LOW",
                _ => "MINIMAL",
            }
            .to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ShipStats;

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
            ..ZkillStats::default()
        });
        assert_eq!(calculate_threat_level(&stats), "MINIMAL");
    }

    #[test]
    fn threat_scales_with_activity() {
        // Modest killer: log10(100)*10 = 20, k/d capped contribution.
        let modest = Some(ZkillStats {
            ships_destroyed: 100,
            ships_lost: 100,
            ..ZkillStats::default()
        });
        assert_eq!(calculate_threat_level(&modest), "LOW");

        // Heavy hitter: high k/d, solo kills, danger ratio, active pvp.
        let dangerous = Some(ZkillStats {
            ships_destroyed: 1000,
            ships_lost: 50,
            solo_kills: 100,
            danger_ratio: 90.0,
            active_pvp_kills: 60,
            ..ZkillStats::default()
        });
        assert_eq!(calculate_threat_level(&dangerous), "EXTREME");
    }

    #[test]
    fn kd_ratio_contribution_is_capped() {
        // 10 kills, 0 losses: kd = 10 (capped), score = 10 + 50 = 60 => HIGH.
        let stats = Some(ZkillStats {
            ships_destroyed: 10,
            ships_lost: 0,
            ..ZkillStats::default()
        });
        assert_eq!(calculate_threat_level(&stats), "HIGH");

        // Same but 10000 kills, 0 losses: log10 grows, kd still capped at 10.
        let stats = Some(ZkillStats {
            ships_destroyed: 10000,
            ships_lost: 0,
            ..ZkillStats::default()
        });
        assert_eq!(calculate_threat_level(&stats), "EXTREME");
    }
}
