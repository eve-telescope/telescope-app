import type {
    EntityType,
    IntelAnnotation,
    IntelEntry,
    IntelEntryDetail,
} from '../types'

export const PRESET_ANNOTATION_TAGS: Array<{ tag: string; color: string }> = [
    { tag: 'HOSTILE', color: '#FF3B3B' },
    { tag: 'SCOUT', color: '#00D4FF' },
    { tag: 'SPY', color: '#8B5CF6' },
    { tag: 'FRIENDLY', color: '#00FF88' },
    { tag: 'ALLY', color: '#34D399' },
    { tag: 'NEUTRAL', color: '#FFD93D' },
]

const TAG_DELIMITER = ' | '

export function normalizeAnnotationTag(tag: string): string {
    return tag.trim().replace(/\s+/g, ' ').toUpperCase()
}

export function parseAnnotationTags(label: string): string[] {
    return label.split('|').map(normalizeAnnotationTag).filter(Boolean)
}

export function serializeAnnotationTags(tags: string[]): string {
    return Array.from(
        new Set(tags.map(normalizeAnnotationTag).filter(Boolean))
    ).join(TAG_DELIMITER)
}

export function getPresetAnnotationColor(tag: string): string | null {
    const normalized = normalizeAnnotationTag(tag)
    return (
        PRESET_ANNOTATION_TAGS.find((preset) => preset.tag === normalized)
            ?.color ?? null
    )
}

export function getAnnotationColor(
    tags: string[],
    fallback?: string | null
): string | null {
    for (const tag of tags) {
        const color = getPresetAnnotationColor(tag)
        if (color) {
            return color
        }
    }

    return fallback ?? null
}

export function mapIntelEntryToAnnotation(
    entry: IntelEntry | IntelEntryDetail,
    networkId?: number,
    networkName?: string
): IntelAnnotation {
    const tags = parseAnnotationTags(entry.label ?? '')
    return {
        id: entry.id,
        networkId:
            networkId ??
            ('intel_network_id' in entry ? entry.intel_network_id : 0),
        networkName:
            networkName ?? ('network_name' in entry ? entry.network_name : ''),
        targetType: entry.entity_type as EntityType,
        targetId: entry.entity_id,
        targetName: entry.entity_name,
        tags,
        note: entry.notes ?? null,
        color: getAnnotationColor(tags, entry.color ?? null),
        createdBy:
            'added_by' in entry && entry.added_by
                ? {
                      id: entry.added_by.id,
                      characterName: entry.added_by.character_name,
                  }
                : null,
    }
}

export function getAnnotationTargetKey(
    targetType: EntityType,
    targetId: number
): string {
    return `${targetType}:${targetId}`
}

export function formatAnnotationScope(scope: EntityType): string {
    switch (scope) {
        case 'character':
            return 'CHAR'
        case 'corporation':
            return 'CORP'
        case 'alliance':
            return 'ALLY'
    }
}
