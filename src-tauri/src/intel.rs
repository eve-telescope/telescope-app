use crate::models::ZkillStats;

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
