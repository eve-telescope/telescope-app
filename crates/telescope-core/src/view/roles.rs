//! Intel roles a pilot flies, derived from their top zKillboard ships. Each
//! role is drawn with the game's overview bracket for its hull, tinted by
//! category, in place of the old text flags.

use super::dscan_view::Hull;
use crate::models::PilotIntel;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    Titan,
    Supercarrier,
    Dreadnought,
    Carrier,
    ForceAuxiliary,
    BlackOps,
    Marauder,
    Recon,
    HeavyInterdictor,
    Interdictor,
    CommandShip,
    StrategicCruiser,
    StealthBomber,
    CovertOps,
    Logistics,
    Cyno,
    Solo,
}

impl Role {
    pub const ALL: [Role; 17] = [
        Role::Titan,
        Role::Supercarrier,
        Role::Dreadnought,
        Role::Carrier,
        Role::ForceAuxiliary,
        Role::BlackOps,
        Role::Marauder,
        Role::Recon,
        Role::HeavyInterdictor,
        Role::Interdictor,
        Role::CommandShip,
        Role::StrategicCruiser,
        Role::StealthBomber,
        Role::CovertOps,
        Role::Logistics,
        Role::Cyno,
        Role::Solo,
    ];

    pub fn from_label(label: &str) -> Option<Role> {
        Role::ALL.into_iter().find(|role| role.label() == label)
    }

    /// Filter and tooltip label. Also the tag text in sidebar chips.
    pub fn label(self) -> &'static str {
        match self {
            Role::Titan => "TITAN",
            Role::Supercarrier => "SUPER",
            Role::Dreadnought => "DREAD",
            Role::Carrier => "CARRIER",
            Role::ForceAuxiliary => "FAX",
            Role::BlackOps => "BLOPS",
            Role::Marauder => "MARAUDER",
            Role::Recon => "RECON",
            Role::HeavyInterdictor => "HIC",
            Role::Interdictor => "DICTOR",
            Role::CommandShip => "COMMAND",
            Role::StrategicCruiser => "T3C",
            Role::StealthBomber => "BOMBER",
            Role::CovertOps => "COVOPS",
            Role::Logistics => "LOGI",
            Role::Cyno => "CYNO",
            Role::Solo => "SOLO",
        }
    }

    /// Lower is more dangerous; icons are shown in this order.
    pub fn danger_rank(self) -> u8 {
        match self {
            Role::Recon => 0,
            Role::Marauder => 1,
            Role::Titan => 2,
            Role::Supercarrier => 3,
            Role::BlackOps => 4,
            Role::Dreadnought => 5,
            Role::HeavyInterdictor => 6,
            Role::Carrier => 7,
            Role::ForceAuxiliary => 8,
            Role::Interdictor => 9,
            Role::StealthBomber => 10,
            Role::StrategicCruiser => 11,
            Role::CommandShip => 12,
            Role::Cyno => 13,
            Role::CovertOps => 14,
            Role::Logistics => 15,
            Role::Solo => 16,
        }
    }

    /// Red is kept for the most dangerous hulls; every other role gets its
    /// own hue so icons stay distinguishable at a glance.
    pub fn color(self) -> &'static str {
        match self {
            Role::Recon | Role::Marauder => "#FF3B5C",
            Role::Titan | Role::Supercarrier => "#E879F9",
            Role::Dreadnought | Role::Carrier => "#F59E0B",
            Role::ForceAuxiliary => "#2DD4BF",
            Role::BlackOps => "#6366F1",
            Role::StealthBomber => "#8B5CF6",
            Role::CovertOps => "#60A5FA",
            Role::Cyno => "#A855F7",
            Role::HeavyInterdictor => "#FB923C",
            Role::Interdictor => "#FACC15",
            Role::CommandShip => "#38BDF8",
            Role::StrategicCruiser => "#22D3EE",
            Role::Logistics => "#44DDAA",
            Role::Solo => "#A3E635",
        }
    }

    /// Bracket icon stem (see `Hull::bracket`), `None` for text-only roles.
    pub fn icon(self) -> Option<&'static str> {
        Some(match self {
            Role::Titan => Hull::Titan.bracket(),
            Role::Supercarrier => Hull::Supercarrier.bracket(),
            Role::Dreadnought => Hull::Dreadnought.bracket(),
            Role::Carrier => Hull::Carrier.bracket(),
            Role::ForceAuxiliary => Hull::ForceAuxiliary.bracket(),
            Role::BlackOps | Role::Marauder => Hull::Battleship.bracket(),
            Role::Recon | Role::HeavyInterdictor | Role::StrategicCruiser | Role::Logistics => {
                Hull::Cruiser.bracket()
            }
            Role::CommandShip => Hull::Battlecruiser.bracket(),
            Role::Interdictor => Hull::Destroyer.bracket(),
            Role::StealthBomber | Role::CovertOps => Hull::Frigate.bracket(),
            Role::Cyno => "cynosuralfield",
            Role::Solo => return None,
        })
    }

    /// The role of an SDE ship group id, `None` for ordinary hulls.
    pub fn for_group(group_id: i64) -> Option<Role> {
        Some(match group_id {
            30 => Role::Titan,
            659 => Role::Supercarrier,
            485 | 4594 => Role::Dreadnought,
            547 | 5120 => Role::Carrier,
            1538 => Role::ForceAuxiliary,
            898 => Role::BlackOps,
            900 => Role::Marauder,
            833 | 906 => Role::Recon,
            894 => Role::HeavyInterdictor,
            541 => Role::Interdictor,
            540 => Role::CommandShip,
            963 => Role::StrategicCruiser,
            834 => Role::StealthBomber,
            830 => Role::CovertOps,
            832 => Role::Logistics,
            _ => return None,
        })
    }
}

/// The pilot's roles, most dangerous first. Cyno and solo come from their
/// zKillboard flags.
pub fn pilot_roles(pilot: &PilotIntel) -> Vec<Role> {
    let mut order: Vec<Role> = Vec::new();
    // Recent appearances from top ships, plus all-time losses per group so
    // hulls that dropped out of the recent top list still count.
    let recent = pilot
        .zkill
        .iter()
        .flat_map(|z| &z.top_ships)
        .map(|ship| (ship.group_id, ship.kills + ship.losses));
    let all_time = pilot
        .zkill
        .iter()
        .flat_map(|z| &z.lost_groups)
        .map(|group| (group.group_id, group.losses));
    for (group_id, appearances) in recent.chain(all_time) {
        let Some(role) = Role::for_group(group_id) else {
            continue;
        };
        if appearances > 0 && !order.contains(&role) {
            order.push(role);
        }
    }
    if pilot.flags.is_cyno {
        order.push(Role::Cyno);
    }
    if pilot.flags.is_solo {
        order.push(Role::Solo);
    }
    order.sort_by_key(|role| role.danger_rank());
    order
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ShipStats, ZkillStats};
    use crate::view::test_support::pilot;

    fn ship(group_id: i64, kills: i64) -> ShipStats {
        ShipStats {
            ship_type_id: group_id * 10,
            ship_name: String::new(),
            group_id,
            group_name: String::new(),
            kills,
            losses: 0,
        }
    }

    fn flying(ships: Vec<ShipStats>) -> PilotIntel {
        let mut p = pilot(1, "p");
        p.zkill = Some(ZkillStats {
            top_ships: ships,
            ..Default::default()
        });
        p
    }

    #[test]
    fn roles_sort_by_danger_and_merge_recon_groups() {
        let p = flying(vec![
            ship(541, 90),
            ship(900, 50),
            ship(833, 1),
            ship(906, 3),
            ship(26, 200),
        ]);
        assert_eq!(
            pilot_roles(&p),
            vec![Role::Recon, Role::Marauder, Role::Interdictor]
        );
    }

    #[test]
    fn ships_never_flown_do_not_count() {
        let p = flying(vec![ship(30, 0)]);
        assert!(pilot_roles(&p).is_empty());
    }

    #[test]
    fn all_time_losses_add_roles() {
        let mut p = flying(vec![ship(26, 40)]);
        if let Some(z) = &mut p.zkill {
            z.lost_groups = vec![crate::models::GroupLosses {
                group_id: 485,
                losses: 2,
            }];
        }
        assert_eq!(pilot_roles(&p), vec![Role::Dreadnought]);
    }

    #[test]
    fn cyno_and_solo_come_last() {
        let mut p = flying(vec![ship(898, 2)]);
        p.flags.is_cyno = true;
        p.flags.is_solo = true;
        assert_eq!(
            pilot_roles(&p),
            vec![Role::BlackOps, Role::Cyno, Role::Solo]
        );
    }

    #[test]
    fn icons_use_the_hull_bracket() {
        assert_eq!(Role::Marauder.icon(), Some("battleship"));
        assert_eq!(Role::Recon.icon(), Some("cruiser"));
        assert_eq!(Role::Solo.icon(), None);
    }

    #[test]
    fn labels_round_trip() {
        for role in Role::ALL {
            assert_eq!(Role::from_label(role.label()), Some(role));
        }
        assert_eq!(Role::from_label("HOSTILE"), None);
    }
}
