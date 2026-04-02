import { computed, type Ref } from 'vue'
import type { PilotIntel } from '../types'
import { getPilotTags } from '../utils/pilotTags'

export interface ThreatCounts {
    extreme: number
    high: number
    moderate: number
    low: number
    minimal: number
}

export interface TagCount {
    tag: string
    color: string | null
    count: number
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

    const tagCounts = computed<TagCount[]>(() => {
        const counts = new Map<
            string,
            { color: string | null; count: number }
        >()
        for (const p of pilots.value) {
            const seen = new Set<string>()
            for (const t of getPilotTags(p)) {
                if (seen.has(t.tag)) continue
                seen.add(t.tag)
                const existing = counts.get(t.tag)
                if (existing) {
                    existing.count++
                } else {
                    counts.set(t.tag, { color: t.color, count: 1 })
                }
            }
        }
        return [...counts.entries()]
            .map(([tag, { color, count }]) => ({ tag, color, count }))
            .sort((a, b) => b.count - a.count)
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
