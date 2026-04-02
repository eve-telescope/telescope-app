import type { PilotIntel } from '../types'
import { resolvePilotAnnotations } from '../stores/intel'
import { getFlagLabels } from './intel'

export interface PilotTag {
    key: string
    tag: string
    color: string | null
}

const FLAG_COLORS: Record<string, string> = {
    SUPER: '#F43F5E',
    CAPITAL: '#F59E0B',
    'BLACK OPS': '#6366F1',
    RECON: '#14B8A6',
    CYNO: '#A855F7',
    SOLO: '#38BDF8',
}

export function getPilotTags(pilot: PilotIntel): PilotTag[] {
    const tags: PilotTag[] = []
    const seen = new Set<string>()

    // Zkill flags
    for (const flag of getFlagLabels(pilot.flags)) {
        seen.add(flag)
        tags.push({
            key: `flag:${flag}`,
            tag: flag,
            color: FLAG_COLORS[flag] ?? null,
        })
    }

    // Intel annotation tags
    for (const match of resolvePilotAnnotations(pilot)) {
        for (const tag of match.annotation.tags) {
            if (!seen.has(tag)) {
                seen.add(tag)
                tags.push({
                    key: `${match.key}:${tag}`,
                    tag,
                    color: match.annotation.color,
                })
            }
        }
    }

    return tags
}

export function getPilotTagStrings(pilot: PilotIntel): string[] {
    return getPilotTags(pilot).map((t) => t.tag)
}
