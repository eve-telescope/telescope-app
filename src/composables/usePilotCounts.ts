import { computed, type Ref } from 'vue'
import type { PilotIntel } from '../types'

export interface ThreatCounts {
    extreme: number
    high: number
    moderate: number
    low: number
    minimal: number
}

export interface TagCounts {
    cyno: number
    recon: number
    blops: number
    capital: number
    super: number
    solo: number
}

export interface GroupCount {
    ticker: string
    name: string
    count: number
}

export function usePilotCounts(pilots: Ref<PilotIntel[]>) {
    const threatCounts = computed<ThreatCounts>(() => {
        const counts = { extreme: 0, high: 0, moderate: 0, low: 0, minimal: 0 }
        for (const p of pilots.value) {
            const level = p.threat_level.toLowerCase() as keyof ThreatCounts
            if (level in counts) counts[level]++
        }
        return counts
    })

    const tagCounts = computed<TagCounts>(() => {
        const counts = {
            cyno: 0,
            recon: 0,
            blops: 0,
            capital: 0,
            super: 0,
            solo: 0,
        }
        for (const p of pilots.value) {
            if (p.flags.is_super) counts.super++
            else if (p.flags.is_capital) counts.capital++
            if (p.flags.is_blops) counts.blops++
            if (p.flags.is_recon) counts.recon++
            if (p.flags.is_cyno) counts.cyno++
            if (p.flags.is_solo) counts.solo++
        }
        return counts
    })

    const corpCounts = computed<GroupCount[]>(() => {
        const counts = new Map<
            string,
            { ticker: string; name: string; count: number }
        >()
        for (const p of pilots.value) {
            const ticker = p.character.corporation_ticker
            if (ticker) {
                const existing = counts.get(ticker)
                if (existing) {
                    existing.count++
                } else {
                    counts.set(ticker, {
                        ticker,
                        name: p.character.corporation_name || ticker,
                        count: 1,
                    })
                }
            }
        }
        return [...counts.values()].sort((a, b) => b.count - a.count)
    })

    const allianceCounts = computed<GroupCount[]>(() => {
        const counts = new Map<
            string,
            { ticker: string; name: string; count: number }
        >()
        for (const p of pilots.value) {
            const ticker = p.character.alliance_ticker
            if (ticker) {
                const existing = counts.get(ticker)
                if (existing) {
                    existing.count++
                } else {
                    counts.set(ticker, {
                        ticker,
                        name: p.character.alliance_name || ticker,
                        count: 1,
                    })
                }
            }
        }
        return [...counts.values()].sort((a, b) => b.count - a.count)
    })

    return {
        threatCounts,
        tagCounts,
        corpCounts,
        allianceCounts,
    }
}
