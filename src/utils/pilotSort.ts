import type { PilotIntel } from '../types'
import { getKdRatioValue, getPpk as getPpkValue } from './format'

export type SortDirection = 'asc' | 'desc'

// Shared collator: dramatically cheaper than String#localeCompare per
// comparison, and 'base' sensitivity makes name sorts case-insensitive.
const collator = new Intl.Collator(undefined, { sensitivity: 'base' })

// Higher rank = more dangerous; unknown/unrecognized levels rank lowest.
const THREAT_RANK: Record<string, number> = {
    extreme: 5,
    high: 4,
    moderate: 3,
    low: 2,
    minimal: 1,
    unknown: 0,
}

// Caches rank per raw threat string (e.g. 'EXTREME', 'Extreme') so sorting
// doesn't lowercase the same handful of strings once per comparison. The
// set of distinct threat strings is tiny, so this never grows unbounded.
const threatRankCache = new Map<string, number>()

export function getThreatRank(level: string): number {
    let rank = threatRankCache.get(level)
    if (rank === undefined) {
        rank = THREAT_RANK[level.toLowerCase()] || 0
        threatRankCache.set(level, rank)
    }
    return rank
}

export function getTagScore(pilot: PilotIntel): number {
    let score = 0
    if (pilot.flags.is_super) score += 100
    if (pilot.flags.is_capital) score += 50
    if (pilot.flags.is_blops) score += 25
    if (pilot.flags.is_recon) score += 12
    if (pilot.flags.is_cyno) score += 6
    if (pilot.flags.is_solo) score += 3
    return score
}

export function getKdRatio(pilot: PilotIntel): number {
    if (!pilot.zkill) return 0
    return getKdRatioValue(pilot.zkill.ships_destroyed, pilot.zkill.ships_lost)
}

export function getPpk(pilot: PilotIntel): number {
    if (!pilot.zkill) return 0
    return getPpkValue(
        pilot.zkill.points_destroyed,
        pilot.zkill.ships_destroyed
    )
}

/**
 * Compare two pilots for the given sort key (ascending). 'corporation' and
 * 'alliance' compare by full name (main table headers); 'corp' and 'ally'
 * compare by ticker (compact overlay headers). Unknown keys compare equal.
 */
export function comparePilots(
    a: PilotIntel,
    b: PilotIntel,
    key: string
): number {
    switch (key) {
        case 'threat':
            return getThreatRank(a.threat_level) - getThreatRank(b.threat_level)
        case 'pilot':
            return collator.compare(a.character.name, b.character.name)
        case 'tags':
            return getTagScore(a) - getTagScore(b)
        case 'corporation':
            return collator.compare(
                a.character.corporation_name || '',
                b.character.corporation_name || ''
            )
        case 'corp':
            return collator.compare(
                a.character.corporation_ticker || '',
                b.character.corporation_ticker || ''
            )
        case 'alliance':
            return collator.compare(
                a.character.alliance_name || '',
                b.character.alliance_name || ''
            )
        case 'ally':
            return collator.compare(
                a.character.alliance_ticker || '',
                b.character.alliance_ticker || ''
            )
        case 'kd':
            return getKdRatio(a) - getKdRatio(b)
        case 'isk':
            return (a.zkill?.isk_destroyed || 0) - (b.zkill?.isk_destroyed || 0)
        case 'ppk':
            return getPpk(a) - getPpk(b)
        case 'cpk':
            return (a.zkill?.avg_attackers || 0) - (b.zkill?.avg_attackers || 0)
        case 'active':
            return (
                (a.zkill?.active_pvp_kills || 0) -
                (b.zkill?.active_pvp_kills || 0)
            )
        case 'danger':
            return (a.zkill?.danger_ratio || 0) - (b.zkill?.danger_ratio || 0)
        default:
            return 0
    }
}

export function sortPilots(
    pilots: readonly PilotIntel[],
    key: string,
    direction: SortDirection
): PilotIntel[] {
    return [...pilots].sort((a, b) => {
        const comparison = comparePilots(a, b, key)
        return direction === 'asc' ? comparison : -comparison
    })
}
