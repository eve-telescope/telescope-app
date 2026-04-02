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

// Intel Network models

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct IntelNetwork {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub entries_count: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IntelEntry {
    pub id: i64,
    pub intel_network_id: i64,
    #[serde(default)]
    pub network_name: String,
    pub entity_type: String,
    pub entity_id: i64,
    pub entity_name: String,
    pub color: Option<String>,
    pub label: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IntelEntryDetail {
    pub id: i64,
    pub entity_type: String,
    pub entity_id: i64,
    pub entity_name: String,
    pub color: Option<String>,
    pub label: Option<String>,
    pub notes: Option<String>,
    pub added_by: Option<AddedBy>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddedBy {
    pub id: i64,
    pub character_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EntityInfo {
    pub id: i64,
    pub name: String,
    #[serde(rename = "type")]
    pub entity_type: Option<String>,
    pub ticker: Option<String>,
    pub corporation: Option<AffiliationInfo>,
    pub alliance: Option<AffiliationInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AffiliationInfo {
    pub id: i64,
    pub name: String,
    pub ticker: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkAccess {
    pub id: i64,
    pub accessible_type: String,
    pub accessible_id: i64,
    pub permission: String,
    pub is_owner: bool,
    pub expires_at: Option<String>,
    pub entity: Option<EntityInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkDetail {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub entries: Vec<IntelEntryDetail>,
    pub accesses: Vec<NetworkAccess>,
}

// Network scans

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScanSubmitter {
    pub id: i64,
    pub character_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkScan {
    pub id: i64,
    pub scan_type: String,
    pub raw_text: String,
    pub solar_system: Option<String>,
    pub created_at: String,
    pub submitted_by: Option<ScanSubmitter>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaginatedScans {
    pub data: Vec<NetworkScan>,
    pub current_page: i64,
    pub last_page: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub id: i64,
    pub name: String,
    pub category: String,
    pub ticker: Option<String>,
    pub corporation: Option<AffiliationInfo>,
    pub alliance: Option<AffiliationInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SdeStatus {
    pub build_number: Option<i64>,
    pub latest_build_number: Option<i64>,
    pub ready: bool,
    pub updating: bool,
    pub last_error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScanTypeIndexEntry {
    pub type_id: i64,
    pub type_name: String,
    pub group_id: i64,
    pub group_name: String,
    pub category_id: i64,
    pub category_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DscanEntry {
    pub type_id: Option<i64>,
    pub name: String,
    pub type_name: String,
    pub distance: Option<String>,
    pub group_name: Option<String>,
    pub category_name: Option<String>,
    pub is_ship: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DscanParseResult {
    pub total_rows: usize,
    pub ship_count: usize,
    pub entries: Vec<DscanEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn network_access_deserializes_simple_type() {
        let data = json!({
            "id": 1,
            "accessible_type": "character",
            "accessible_id": 12345,
            "permission": "viewer",
            "is_owner": false,
            "expires_at": null,
            "entity": null
        });
        let access: NetworkAccess = serde_json::from_value(data).unwrap();
        assert_eq!(access.accessible_type, "character");
        assert_eq!(access.accessible_id, 12345);
        assert!(!access.is_owner);
    }

    #[test]
    fn network_access_with_entity_info() {
        let data = json!({
            "id": 1,
            "accessible_type": "corporation",
            "accessible_id": 98000001,
            "permission": "member",
            "is_owner": false,
            "expires_at": null,
            "entity": {
                "id": 98000001,
                "name": "Test Corp",
                "type": "corporation",
                "ticker": "TSTC",
                "corporation": null,
                "alliance": { "id": 99000001, "name": "Test Alliance", "ticker": "TSTA" }
            }
        });
        let access: NetworkAccess = serde_json::from_value(data).unwrap();
        let entity = access.entity.unwrap();
        assert_eq!(entity.name, "Test Corp");
        assert_eq!(entity.entity_type.as_deref(), Some("corporation"));
        assert!(entity.alliance.is_some());
    }

    #[test]
    fn intel_entry_nullable_color_and_label() {
        let data = json!({
            "id": 1,
            "intel_network_id": 1,
            "entity_type": "character",
            "entity_id": 12345,
            "entity_name": "Test Pilot",
            "color": null,
            "label": null,
            "notes": "Just a note"
        });
        let entry: IntelEntry = serde_json::from_value(data).unwrap();
        assert!(entry.color.is_none());
        assert!(entry.label.is_none());
        assert_eq!(entry.notes.as_deref(), Some("Just a note"));
    }

    #[test]
    fn intel_entry_with_all_fields() {
        let data = json!({
            "id": 1,
            "intel_network_id": 1,
            "entity_type": "character",
            "entity_id": 12345,
            "entity_name": "Bad Guy",
            "color": "#FF3B3B",
            "label": "HOSTILE | SPY",
            "notes": null
        });
        let entry: IntelEntry = serde_json::from_value(data).unwrap();
        assert_eq!(entry.color.as_deref(), Some("#FF3B3B"));
        assert_eq!(entry.label.as_deref(), Some("HOSTILE | SPY"));
    }

    #[test]
    fn intel_entry_default_network_name() {
        let data = json!({
            "id": 1,
            "intel_network_id": 1,
            "entity_type": "character",
            "entity_id": 12345,
            "entity_name": "Test",
            "color": null,
            "label": null,
            "notes": null
        });
        let entry: IntelEntry = serde_json::from_value(data).unwrap();
        assert_eq!(entry.network_name, "");
    }

    #[test]
    fn search_result_deserializes() {
        let data = json!({
            "id": 12345,
            "name": "Test Pilot",
            "category": "character",
            "ticker": null,
            "corporation": { "id": 98000001, "name": "Corp", "ticker": "CRP" },
            "alliance": null
        });
        let result: SearchResult = serde_json::from_value(data).unwrap();
        assert_eq!(result.category, "character");
        assert!(result.corporation.is_some());
        assert!(result.alliance.is_none());
    }

    #[test]
    fn network_scan_deserializes() {
        let data = json!({
            "id": 1,
            "scan_type": "local",
            "raw_text": "Pilot One\nPilot Two",
            "solar_system": null,
            "created_at": "2026-04-01T10:00:00+00:00",
            "submitted_by": { "id": 12345, "character_name": "Scanner" }
        });
        let scan: NetworkScan = serde_json::from_value(data).unwrap();
        assert_eq!(scan.scan_type, "local");
        assert!(scan.submitted_by.is_some());
        assert_eq!(scan.submitted_by.unwrap().character_name, "Scanner");
    }

    #[test]
    fn paginated_scans_deserializes() {
        let data = json!({
            "data": [{
                "id": 1,
                "scan_type": "dscan",
                "raw_text": "12345\tShip\tType\t100km",
                "solar_system": "Jita",
                "created_at": "2026-04-01T10:00:00+00:00",
                "submitted_by": null
            }],
            "current_page": 1,
            "last_page": 3
        });
        let result: PaginatedScans = serde_json::from_value(data).unwrap();
        assert_eq!(result.data.len(), 1);
        assert_eq!(result.current_page, 1);
        assert_eq!(result.last_page, 3);
        assert_eq!(result.data[0].solar_system.as_deref(), Some("Jita"));
    }

    #[test]
    fn entity_info_type_renames_correctly() {
        let data = json!({
            "id": 1,
            "name": "Test",
            "type": "alliance",
            "ticker": "TST",
            "corporation": null,
            "alliance": null
        });
        let info: EntityInfo = serde_json::from_value(data).unwrap();
        assert_eq!(info.entity_type.as_deref(), Some("alliance"));
    }
}
