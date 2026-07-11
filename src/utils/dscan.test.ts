import { describe, it, expect } from 'vitest'
import {
    Anchor,
    Eye,
    HeartPulse,
    Ship,
    ShieldHalf,
    Truck,
} from 'lucide-vue-next'
import { barWidth, bucketByType, classIcon, countByClass } from './dscan'
import type { DscanEntry } from '../types'

function makeEntry(overrides: Partial<DscanEntry> = {}): DscanEntry {
    return {
        type_id: 1,
        name: 'Some Pilot',
        type_name: 'Sabre',
        distance: null,
        group_name: 'Interdictor',
        category_name: 'Ship',
        is_ship: true,
        ...overrides,
    }
}

describe('bucketByType', () => {
    it('counts entries per type name', () => {
        const entries = [
            makeEntry({ type_name: 'Sabre' }),
            makeEntry({ type_name: 'Sabre' }),
            makeEntry({ type_name: 'Drake', group_name: 'Battlecruiser' }),
        ]
        const buckets = bucketByType(entries, (e) => e.group_name ?? 'Unknown')
        expect(buckets).toHaveLength(2)
        expect(buckets[0]).toMatchObject({ type_name: 'Sabre', count: 2 })
        expect(buckets[1]).toMatchObject({ type_name: 'Drake', count: 1 })
    })

    it('sorts by count descending, then by name for ties', () => {
        const entries = [
            makeEntry({ type_name: 'Zealot' }),
            makeEntry({ type_name: 'Atron' }),
            makeEntry({ type_name: 'Drake' }),
            makeEntry({ type_name: 'Drake' }),
        ]
        const buckets = bucketByType(entries, () => '')
        expect(buckets.map((b) => b.type_name)).toEqual([
            'Drake',
            'Atron',
            'Zealot',
        ])
    })

    it('uses the first entry of a type for subtitle and type_id', () => {
        const entries = [
            makeEntry({ type_name: 'Sabre', type_id: 22456 }),
            makeEntry({ type_name: 'Sabre', type_id: 99999 }),
        ]
        const buckets = bucketByType(entries, (e) => `id:${e.type_id}`)
        expect(buckets[0].type_id).toBe(22456)
        expect(buckets[0].subtitle).toBe('id:22456')
    })

    it('returns empty for no entries', () => {
        expect(bucketByType([], () => '')).toEqual([])
    })
})

describe('countByClass', () => {
    it('counts entries per group name with unknown fallback', () => {
        const entries = [
            makeEntry({ group_name: 'Interdictor' }),
            makeEntry({ group_name: 'Interdictor' }),
            makeEntry({ group_name: null }),
        ]
        const counts = countByClass(entries)
        expect(counts).toEqual([
            { name: 'Interdictor', count: 2 },
            { name: 'Unknown class', count: 1 },
        ])
    })

    it('breaks count ties by name', () => {
        const entries = [
            makeEntry({ group_name: 'Frigate' }),
            makeEntry({ group_name: 'Battleship' }),
        ]
        expect(countByClass(entries).map((c) => c.name)).toEqual([
            'Battleship',
            'Frigate',
        ])
    })
})

describe('classIcon', () => {
    it('matches battlecruiser before battleship keywords', () => {
        expect(classIcon('Combat Battlecruiser')).toBe(ShieldHalf)
        expect(classIcon('Attack Battlecruiser')).toBe(ShieldHalf)
    })

    it('matches covert/recon to the same intel icon family', () => {
        expect(classIcon('Combat Recon Ship')).toBe(Eye)
        expect(classIcon('Covert Ops')).toBe(Eye)
    })

    it('prefers capital over industrial for capital industrials', () => {
        // 'capital' appears earlier in the rule list than 'industrial'
        expect(classIcon('Capital Industrial Ship')).toBe(Anchor)
        expect(classIcon('Industrial Command Ship')).toBe(Truck)
    })

    it('matches logistics cruisers as logistics, not generic', () => {
        expect(classIcon('Logistics')).toBe(HeartPulse)
        expect(classIcon('Logistics Frigate')).toBe(HeartPulse)
    })

    it('is case-insensitive', () => {
        expect(classIcon('TITAN')).toBe(Anchor)
    })

    it('falls back to the generic ship icon', () => {
        expect(classIcon('Heavy Assault Cruiser')).toBe(Ship)
        expect(classIcon('')).toBe(Ship)
    })
})

describe('barWidth', () => {
    it('returns 0% when the maximum is zero', () => {
        expect(barWidth(5, 0, 6)).toBe('0%')
    })

    it('scales relative to the maximum', () => {
        expect(barWidth(50, 100, 6)).toBe('50%')
        expect(barWidth(100, 100, 6)).toBe('100%')
    })

    it('clamps small values to the minimum percent', () => {
        expect(barWidth(1, 1000, 6)).toBe('6%')
        expect(barWidth(1, 1000, 4)).toBe('4%')
    })
})
