import { ref, computed, type Ref, type WritableComputedRef } from 'vue'
import type { PilotIntel } from '../types'

const THREAT_ORDER: Record<string, number> = {
    extreme: 5,
    high: 4,
    moderate: 3,
    low: 2,
    minimal: 1,
    unknown: 0,
}

function getTagScore(pilot: PilotIntel): number {
    let score = 0
    if (pilot.flags.is_super) score += 100
    if (pilot.flags.is_capital) score += 50
    if (pilot.flags.is_blops) score += 25
    if (pilot.flags.is_recon) score += 12
    if (pilot.flags.is_cyno) score += 6
    if (pilot.flags.is_solo) score += 3
    return score
}

function getKdRatio(pilot: PilotIntel): number {
    if (!pilot.zkill) return 0
    if (pilot.zkill.ships_lost > 0) {
        return pilot.zkill.ships_destroyed / pilot.zkill.ships_lost
    }
    return pilot.zkill.ships_destroyed
}

function getPpk(pilot: PilotIntel): number {
    if (!pilot.zkill || pilot.zkill.ships_destroyed === 0) return 0
    return pilot.zkill.points_destroyed / pilot.zkill.ships_destroyed
}

export interface UsePilotSortOptions {
    sortKey?: Ref<string> | WritableComputedRef<string>
    sortDirection?: Ref<'asc' | 'desc'> | WritableComputedRef<'asc' | 'desc'>
}

export function usePilotSort(
    pilots: Ref<PilotIntel[]>,
    options?: UsePilotSortOptions
) {
    const sortKey = options?.sortKey ?? ref<string>('threat')
    const sortDirection = options?.sortDirection ?? ref<'asc' | 'desc'>('desc')

    function handleSort(key: string) {
        if (sortKey.value === key) {
            sortDirection.value = sortDirection.value === 'asc' ? 'desc' : 'asc'
        } else {
            sortKey.value = key
            sortDirection.value = 'desc'
        }
    }

    const sortedPilots = computed(() => {
        const sorted = [...pilots.value].sort((a, b) => {
            let comparison = 0

            switch (sortKey.value) {
                case 'threat':
                    comparison =
                        (THREAT_ORDER[a.threat_level.toLowerCase()] || 0) -
                        (THREAT_ORDER[b.threat_level.toLowerCase()] || 0)
                    break
                case 'pilot':
                    comparison = a.character.name.localeCompare(
                        b.character.name
                    )
                    break
                case 'tags':
                    comparison = getTagScore(a) - getTagScore(b)
                    break
                case 'corporation':
                    comparison = (
                        a.character.corporation_name || ''
                    ).localeCompare(b.character.corporation_name || '')
                    break
                case 'corp':
                    comparison = (
                        a.character.corporation_ticker || ''
                    ).localeCompare(b.character.corporation_ticker || '')
                    break
                case 'alliance':
                    comparison = (
                        a.character.alliance_name || ''
                    ).localeCompare(b.character.alliance_name || '')
                    break
                case 'ally':
                    comparison = (
                        a.character.alliance_ticker || ''
                    ).localeCompare(b.character.alliance_ticker || '')
                    break
                case 'kd':
                    comparison = getKdRatio(a) - getKdRatio(b)
                    break
                case 'isk':
                    comparison =
                        (a.zkill?.isk_destroyed || 0) -
                        (b.zkill?.isk_destroyed || 0)
                    break
                case 'ppk':
                    comparison = getPpk(a) - getPpk(b)
                    break
                case 'cpk':
                    comparison =
                        (a.zkill?.avg_attackers || 0) -
                        (b.zkill?.avg_attackers || 0)
                    break
                case 'active':
                    comparison =
                        (a.zkill?.active_pvp_kills || 0) -
                        (b.zkill?.active_pvp_kills || 0)
                    break
                case 'danger':
                    comparison =
                        (a.zkill?.danger_ratio || 0) -
                        (b.zkill?.danger_ratio || 0)
                    break
            }

            return sortDirection.value === 'asc' ? comparison : -comparison
        })

        return sorted
    })

    return {
        sortKey,
        sortDirection,
        handleSort,
        sortedPilots,
    }
}
