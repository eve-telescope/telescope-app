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
