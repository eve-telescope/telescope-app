import {
    Anchor,
    Bomb,
    CircleDot,
    Crosshair,
    Eye,
    HeartPulse,
    Pickaxe,
    Rocket,
    Satellite,
    Shield,
    ShieldHalf,
    Ship,
    Swords,
    Truck,
    Zap,
    type LucideProps,
} from 'lucide-vue-next'
import type { FunctionalComponent } from 'vue'
import type { DscanEntry } from '../types'

export type ClassIcon = FunctionalComponent<LucideProps>

// Map an EVE ship class (group name) to a fitting icon. Rules are checked in
// order, so more specific classes must come before broader ones (e.g.
// "battlecruiser" before "cruiser").
export const CLASS_ICON_RULES: [string, ClassIcon][] = [
    ['logistic', HeartPulse],
    ['force auxiliary', HeartPulse],
    ['capsule', CircleDot],
    ['interceptor', Zap],
    ['interdictor', Crosshair],
    ['covert', Eye],
    ['recon', Eye],
    ['stealth', Bomb],
    ['bomber', Bomb],
    ['electronic', Satellite],
    ['titan', Anchor],
    ['supercarrier', Anchor],
    ['carrier', Anchor],
    ['dreadnought', Anchor],
    ['capital', Anchor],
    ['freighter', Truck],
    ['industrial', Truck],
    ['transport', Truck],
    ['hauler', Truck],
    ['mining', Pickaxe],
    ['barge', Pickaxe],
    ['exhumer', Pickaxe],
    ['command', Swords],
    ['destroyer', Swords],
    ['battlecruiser', ShieldHalf],
    ['marauder', Shield],
    ['battleship', Shield],
    ['frigate', Rocket],
    ['shuttle', Rocket],
]

export function classIcon(name: string): ClassIcon {
    const lower = name.toLowerCase()
    for (const [keyword, icon] of CLASS_ICON_RULES) {
        if (lower.includes(keyword)) {
            return icon
        }
    }
    return Ship
}

export interface TypeBucket {
    type_id: number | null
    type_name: string
    subtitle: string
    count: number
}

/**
 * Group scan entries by type name, counting instances. The subtitle comes
 * from the first entry of each type. Sorted by count descending, then name.
 */
export function bucketByType(
    entries: DscanEntry[],
    subtitleOf: (e: DscanEntry) => string
): TypeBucket[] {
    const map = new Map<string, TypeBucket>()

    for (const entry of entries) {
        const bucket = map.get(entry.type_name)
        if (bucket) {
            bucket.count += 1
        } else {
            map.set(entry.type_name, {
                type_id: entry.type_id,
                type_name: entry.type_name,
                subtitle: subtitleOf(entry),
                count: 1,
            })
        }
    }

    return [...map.values()].sort(
        (a, b) => b.count - a.count || a.type_name.localeCompare(b.type_name)
    )
}

export interface ClassCount {
    name: string
    count: number
}

/**
 * Count scan entries per ship class (group name). Sorted by count
 * descending, then name.
 */
export function countByClass(entries: DscanEntry[]): ClassCount[] {
    const map = new Map<string, number>()
    for (const entry of entries) {
        const key = entry.group_name ?? 'Unknown class'
        map.set(key, (map.get(key) ?? 0) + 1)
    }
    return [...map.entries()]
        .map(([name, count]) => ({ name, count }))
        .sort((a, b) => b.count - a.count || a.name.localeCompare(b.name))
}

/** Bar width as a CSS percentage, clamped to a minimum for visibility. */
export function barWidth(
    count: number,
    max: number,
    minPercent: number
): string {
    if (max === 0) return '0%'
    return `${Math.max(minPercent, (count / max) * 100)}%`
}
