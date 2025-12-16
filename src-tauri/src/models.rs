use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CharacterInfo {
    pub id: i64,
    pub name: String,
    pub corporation_id: Option<i64>,
    pub corporation_name: Option<String>,
    pub corporation_ticker: Option<String>,
    pub alliance_id: Option<i64>,
    pub alliance_name: Option<String>,
    pub alliance_ticker: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ZkillStats {
    pub ships_destroyed: i64,
    pub ships_lost: i64,
    pub isk_destroyed: f64,
    pub isk_lost: f64,
    pub solo_kills: i64,
    pub solo_losses: i64,
    pub danger_ratio: f64,
    pub gang_ratio: f64,
    pub points_destroyed: i64,
    pub active_pvp_kills: i64,
    pub avg_attackers: f64,
    pub top_ships: Vec<ShipStats>,
    pub activity: Option<ActivityHeatmap>,
    pub top_systems: Vec<SystemStats>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShipStats {
    pub ship_type_id: i64,
    pub ship_name: String,
    pub group_id: i64,
    pub group_name: String,
    pub kills: i64,
    pub losses: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActivityHeatmap {
    pub max: i64,
    pub data: Vec<Vec<i64>>, // 7 days x 24 hours
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemStats {
    pub system_id: i64,
    pub system_name: String,
    pub kills: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PilotFlags {
    pub is_cyno: bool,
    pub is_recon: bool,
    pub is_blops: bool,
    pub is_capital: bool,
    pub is_super: bool,
    pub is_solo: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PilotIntel {
    pub character: CharacterInfo,
    pub zkill: Option<ZkillStats>,
    pub threat_level: String,
    pub flags: PilotFlags,
    pub error: Option<String>,
}
