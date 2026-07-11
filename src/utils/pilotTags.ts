import type {
    IntelAnnotation,
    PilotIntel,
    ResolvedIntelAnnotation,
} from '../types'
import {
    annotationsByTargetKey,
    resolvePilotAnnotations,
} from '../stores/intel'
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

// Fallback chip colors for tags without an explicit annotation color.
// Kept as hex literals (not CSS variables) because callers append
// 2-digit alpha suffixes to build inline background/border styles.
export const DEFAULT_TAG_COLOR = '#94A3B8'
export const DEFAULT_TAG_TEXT_COLOR = '#CBD5E1'

// Memoizes the no-explicit-annotations path of getPilotTags. Pilot objects
// are stable across flushes (the accumulator reuses them), so the WeakMap
// key survives re-renders; a replaced pilot object simply misses and is
// recomputed. `index` is the annotationsByTargetKey object identity — the
// index computed produces a fresh object whenever intel entries change, so
// an identity mismatch means the cached tags may be stale.
const tagCache = new WeakMap<
    PilotIntel,
    { index: Record<string, IntelAnnotation[]>; tags: PilotTag[] }
>()

export function getPilotTags(
    pilot: PilotIntel,
    // Callers that already resolved annotations can pass them to avoid
    // resolving twice per pilot.
    resolved?: ResolvedIntelAnnotation[]
): PilotTag[] {
    if (resolved === undefined) {
        // Reading .value here also registers the reactive dependency for
        // callers inside computeds, exactly like the uncached path does.
        const index = annotationsByTargetKey.value
        const cached = tagCache.get(pilot)
        if (cached && cached.index === index) {
            return cached.tags
        }
        const tags = computePilotTags(pilot, resolvePilotAnnotations(pilot))
        tagCache.set(pilot, { index, tags })
        return tags
    }

    return computePilotTags(pilot, resolved)
}

function computePilotTags(
    pilot: PilotIntel,
    resolved: ResolvedIntelAnnotation[]
): PilotTag[] {
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
    for (const match of resolved) {
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
