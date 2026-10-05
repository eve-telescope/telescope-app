use crate::models::{CharacterInfo, PilotFlags, PilotIntel, ZkillStats};

pub fn pilot(id: i64, name: &str) -> PilotIntel {
    PilotIntel {
        character: CharacterInfo {
            id,
            name: name.to_string(),
            corporation_id: None,
            corporation_name: None,
            corporation_ticker: None,
            alliance_id: None,
            alliance_name: None,
            alliance_ticker: None,
        },
        zkill: None,
        threat_level: "unknown".to_string(),
        danger: None,
        flags: PilotFlags::default(),
        error: None,
    }
}

/// A pilot in "Test Corp" [TSTC] / "Test Alliance" [TSTA].
pub fn affiliated_pilot(id: i64, name: &str) -> PilotIntel {
    let mut p = pilot(id, name);
    p.character.corporation_id = Some(98000001);
    p.character.corporation_name = Some("Test Corp".into());
    p.character.corporation_ticker = Some("TSTC".into());
    p.character.alliance_id = Some(99000001);
    p.character.alliance_name = Some("Test Alliance".into());
    p.character.alliance_ticker = Some("TSTA".into());
    p
}

pub fn with_threat(mut p: PilotIntel, threat: &str) -> PilotIntel {
    p.threat_level = threat.to_string();
    p
}

pub fn with_flags(mut p: PilotIntel, f: impl FnOnce(&mut PilotFlags)) -> PilotIntel {
    f(&mut p.flags);
    p
}

pub fn with_zkill(mut p: PilotIntel, f: impl FnOnce(&mut ZkillStats)) -> PilotIntel {
    let mut stats = ZkillStats::default();
    f(&mut stats);
    p.zkill = Some(stats);
    p
}

pub fn names<'a>(pilots: impl IntoIterator<Item = &'a PilotIntel>) -> Vec<&'a str> {
    pilots
        .into_iter()
        .map(|p| p.character.name.as_str())
        .collect()
}

/// A pilot whose top ships include a Force Recon with kills.
pub fn flying_recon(mut p: PilotIntel) -> PilotIntel {
    p.zkill = Some(ZkillStats {
        top_ships: vec![crate::models::ShipStats {
            ship_type_id: 11957,
            ship_name: "Falcon".into(),
            group_id: 833,
            group_name: "Force Recon Ship".into(),
            kills: 1,
            losses: 0,
        }],
        ..Default::default()
    });
    p
}
