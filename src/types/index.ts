export interface ShipStats {
    ship_type_id: number
    ship_name: string
    group_id: number
    group_name: string
    kills: number
    losses: number
}

export interface ActivityHeatmap {
    max: number
    data: number[][] // 7 days x 24 hours
}

export interface SystemStats {
    system_id: number
    system_name: string
    kills: number
}

export interface ZkillStats {
    ships_destroyed: number
    ships_lost: number
    isk_destroyed: number
    isk_lost: number
    solo_kills: number
    solo_losses: number
    danger_ratio: number
    gang_ratio: number
    points_destroyed: number
    active_pvp_kills: number
    avg_attackers: number
    top_ships: ShipStats[]
    activity: ActivityHeatmap | null
    top_systems: SystemStats[]
}

export interface CharacterInfo {
    id: number
    name: string
    corporation_id: number | null
    corporation_name: string | null
    corporation_ticker: string | null
    alliance_id: number | null
    alliance_name: string | null
    alliance_ticker: string | null
}

export interface PilotFlags {
    is_cyno: boolean
    is_recon: boolean
    is_blops: boolean
    is_capital: boolean
    is_super: boolean
    is_solo: boolean
}

export interface PilotIntel {
    character: CharacterInfo
    zkill: ZkillStats | null
    threat_level: string
    flags: PilotFlags
    error: string | null
}

export interface SdeStatus {
    build_number: number | null
    latest_build_number: number | null
    ready: boolean
    updating: boolean
    last_error: string | null
}

export interface DscanEntry {
    type_id: number | null
    name: string
    type_name: string
    distance: string | null
    group_name: string | null
    category_name: string | null
    is_ship: boolean
}

export interface DscanParseResult {
    total_rows: number
    ship_count: number
    entries: DscanEntry[]
}

export type EntityType = 'character' | 'corporation' | 'alliance'

export type PermissionLevel = 'viewer' | 'member' | 'manager'

export type AnnotationScope = EntityType

export interface IntelNetwork {
    id: number
    name: string
    slug: string
    entries_count: number
}

export interface IntelEntry {
    id: number
    intel_network_id: number
    network_name: string
    entity_type: EntityType
    entity_id: number
    entity_name: string
    color: string | null
    label: string | null
    notes: string | null
}

export interface IntelAnnotation {
    id: number
    networkId: number
    networkName: string
    targetType: EntityType
    targetId: number
    targetName: string
    tags: string[]
    note: string | null
    color: string | null
    createdBy: { id: number; characterName: string } | null
}

export interface ResolvedIntelAnnotation {
    key: string
    scope: AnnotationScope
    annotation: IntelAnnotation
}

export interface IntelEntryDetail {
    id: number
    entity_type: EntityType
    entity_id: number
    entity_name: string
    color: string | null
    label: string | null
    notes: string | null
    added_by: { id: number; character_name: string } | null
}

export interface EntityInfo {
    id: number
    name: string
    type: EntityType
    ticker?: string
    corporation?: { id: number; name: string; ticker: string } | null
    alliance?: { id: number; name: string; ticker: string } | null
}

export interface SearchResult {
    id: number
    name: string
    category: EntityType
    ticker?: string
    corporation?: { id: number; name: string; ticker: string } | null
    alliance?: { id: number; name: string; ticker: string } | null
}

export interface NetworkAccess {
    id: number
    accessible_type: string
    accessible_id: number
    permission: PermissionLevel
    is_owner: boolean
    expires_at: string | null
    entity: EntityInfo | null
}

export interface ScanSubmitter {
    id: number
    character_name: string
}

export interface NetworkScan {
    id: number
    scan_type: 'local' | 'dscan'
    raw_text: string
    solar_system: string | null
    created_at: string
    submitted_by: ScanSubmitter | null
}

export interface PaginatedScans {
    data: NetworkScan[]
    current_page: number
    last_page: number
}

export interface NetworkDetail {
    id: number
    name: string
    slug: string
    entries: IntelEntryDetail[]
    accesses: NetworkAccess[]
}
