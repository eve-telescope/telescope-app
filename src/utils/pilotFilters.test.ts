import { describe, it, expect, vi } from 'vitest'

// Mock the intel store so the default getPilotTagStrings path (flags only,
// no annotations) works without Tauri.
vi.mock('../stores/intel', () => ({
    resolvePilotAnnotations: vi.fn(() => []),
    annotationsByTargetKey: {
        get value() {
            return {}
        },
    },
}))

import { filterPilots, hasActiveFilters } from './pilotFilters'
import type { PilotFilterState } from './pilotFilters'
import type { PilotIntel } from '../types'

let nextId = 1

function makePilot(
    overrides: Omit<Partial<PilotIntel>, 'flags'> & {
        name?: string
        flags?: Partial<PilotIntel['flags']>
    } = {}
): PilotIntel {
    const { name, flags, ...rest } = overrides
    return {
        character: {
            id: nextId++,
            name: name ?? 'Pilot',
            corporation_id: 98000001,
            corporation_name: 'Test Corp',
            corporation_ticker: 'TSTC',
            alliance_id: 99000001,
            alliance_name: 'Test Alliance',
            alliance_ticker: 'TSTA',
        },
        zkill: null,
        threat_level: 'unknown',
        flags: {
            is_cyno: false,
            is_recon: false,
            is_blops: false,
            is_capital: false,
            is_super: false,
            is_solo: false,
            ...flags,
        },
        error: null,
        ...rest,
    }
}

describe('hasActiveFilters', () => {
    it('is false for an empty state', () => {
        expect(hasActiveFilters({})).toBe(false)
        expect(
            hasActiveFilters({
                threatFilter: null,
                selectedTags: new Set(),
                corpFilter: null,
                allianceFilter: null,
                selectedCorps: new Set(),
                selectedAlliances: new Set(),
            })
        ).toBe(false)
    })

    it('is true when any category is active', () => {
        expect(hasActiveFilters({ threatFilter: 'high' })).toBe(true)
        expect(hasActiveFilters({ selectedTags: new Set(['CYNO']) })).toBe(true)
        expect(hasActiveFilters({ corpFilter: 'TSTC' })).toBe(true)
        expect(hasActiveFilters({ allianceFilter: 'TSTA' })).toBe(true)
        expect(
            hasActiveFilters({ selectedCorps: new Set(['Test Corp']) })
        ).toBe(true)
        expect(
            hasActiveFilters({ selectedAlliances: new Set(['Test Alliance']) })
        ).toBe(true)
    })
})

describe('filterPilots', () => {
    it('returns the input array unchanged when no filters are active', () => {
        const pilots = [makePilot(), makePilot()]
        expect(filterPilots(pilots, {})).toBe(pilots)
    })

    it('filters by threat level (case-insensitive)', () => {
        const pilots = [
            makePilot({ threat_level: 'Extreme' }),
            makePilot({ threat_level: 'low' }),
        ]
        const result = filterPilots(pilots, { threatFilter: 'extreme' })
        expect(result).toHaveLength(1)
        expect(result[0].threat_level).toBe('Extreme')
    })

    it('filters by tag via getPilotTagStrings (flags)', () => {
        const pilots = [makePilot({ flags: { is_cyno: true } }), makePilot()]
        const result = filterPilots(pilots, {
            selectedTags: new Set(['CYNO']),
        })
        expect(result).toHaveLength(1)
        expect(result[0].flags.is_cyno).toBe(true)
    })

    it('ORs tags within the category', () => {
        const pilots = [
            makePilot({ flags: { is_cyno: true } }),
            makePilot({ flags: { is_recon: true } }),
            makePilot(),
        ]
        const result = filterPilots(pilots, {
            selectedTags: new Set(['CYNO', 'RECON']),
        })
        expect(result).toHaveLength(2)
    })

    it('supports an injected tag resolver', () => {
        const hostile = makePilot({ name: 'Hostile' })
        const pilots = [hostile, makePilot()]
        const result = filterPilots(
            pilots,
            { selectedTags: new Set(['HOSTILE']) },
            (p) => (p === hostile ? ['HOSTILE'] : [])
        )
        expect(result).toEqual([hostile])
    })

    it('filters by single-select corp/alliance ticker', () => {
        const other = makePilot()
        other.character.corporation_ticker = 'OTHR'
        other.character.alliance_ticker = 'OTHA'
        const pilots = [makePilot(), other]

        expect(filterPilots(pilots, { corpFilter: 'OTHR' })).toEqual([other])
        expect(filterPilots(pilots, { allianceFilter: 'OTHA' })).toEqual([
            other,
        ])
    })

    it('filters by multi-select corp names, mapping missing corp to Unknown', () => {
        const noCorp = makePilot()
        noCorp.character.corporation_name = null
        const pilots = [makePilot(), noCorp]

        expect(
            filterPilots(pilots, { selectedCorps: new Set(['Unknown']) })
        ).toEqual([noCorp])
        expect(
            filterPilots(pilots, { selectedCorps: new Set(['Test Corp']) })
        ).toHaveLength(1)
    })

    it('never matches pilots without an alliance in multi-select alliance filter', () => {
        const noAlliance = makePilot()
        noAlliance.character.alliance_name = null
        const pilots = [makePilot(), noAlliance]

        const result = filterPilots(pilots, {
            selectedAlliances: new Set(['Test Alliance']),
        })
        expect(result).toHaveLength(1)
        expect(result[0].character.alliance_name).toBe('Test Alliance')
    })

    it('ANDs across categories', () => {
        const match = makePilot({
            threat_level: 'high',
            flags: { is_cyno: true },
        })
        const wrongThreat = makePilot({
            threat_level: 'low',
            flags: { is_cyno: true },
        })
        const wrongTag = makePilot({ threat_level: 'high' })

        const result = filterPilots([match, wrongThreat, wrongTag], {
            threatFilter: 'high',
            selectedTags: new Set(['CYNO']),
        })
        expect(result).toEqual([match])
    })

    const emptyState: PilotFilterState = {
        threatFilter: null,
        selectedTags: new Set(),
        selectedCorps: new Set(),
        selectedAlliances: new Set(),
    }

    it('treats empty sets and nulls as inactive', () => {
        const pilots = [makePilot(), makePilot()]
        expect(filterPilots(pilots, emptyState)).toBe(pilots)
    })
})
