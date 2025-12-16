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
