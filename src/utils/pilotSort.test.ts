import { describe, it, expect } from 'vitest'
import {
    comparePilots,
    getKdRatio,
    getPpk,
    getTagScore,
    getThreatRank,
    sortPilots,
} from './pilotSort'
import type { PilotIntel, ZkillStats } from '../types'

function makeZkill(overrides: Partial<ZkillStats> = {}): ZkillStats {
    return {
        ships_destroyed: 0,
        ships_lost: 0,
        isk_destroyed: 0,
        isk_lost: 0,
        solo_kills: 0,
        solo_losses: 0,
        danger_ratio: 0,
        gang_ratio: 0,
        points_destroyed: 0,
        active_pvp_kills: 0,
        avg_attackers: 0,
        top_ships: [],
        activity: null,
        top_systems: [],
        ...overrides,
    }
}

interface PilotOverrides {
    name?: string
    threat?: string
    zkill?: ZkillStats | null
    flags?: Partial<PilotIntel['flags']>
    corporation_name?: string | null
    corporation_ticker?: string | null
    alliance_name?: string | null
    alliance_ticker?: string | null
}

let nextId = 1

function makePilot(overrides: PilotOverrides = {}): PilotIntel {
    return {
        character: {
            id: nextId++,
            name: overrides.name ?? 'Test Pilot',
            corporation_id: null,
            corporation_name: overrides.corporation_name ?? null,
            corporation_ticker: overrides.corporation_ticker ?? null,
            alliance_id: null,
            alliance_name: overrides.alliance_name ?? null,
            alliance_ticker: overrides.alliance_ticker ?? null,
        },
        zkill: overrides.zkill ?? null,
        threat_level: overrides.threat ?? 'unknown',
        flags: {
            is_cyno: false,
            is_recon: false,
            is_blops: false,
            is_capital: false,
            is_super: false,
            is_solo: false,
            ...overrides.flags,
        },
        error: null,
    }
}

describe('getThreatRank', () => {
    it('ranks levels from extreme down to unknown', () => {
        expect(getThreatRank('extreme')).toBe(5)
        expect(getThreatRank('high')).toBe(4)
        expect(getThreatRank('moderate')).toBe(3)
        expect(getThreatRank('low')).toBe(2)
        expect(getThreatRank('minimal')).toBe(1)
        expect(getThreatRank('unknown')).toBe(0)
    })

    it('is case-insensitive', () => {
        expect(getThreatRank('EXTREME')).toBe(5)
        expect(getThreatRank('Unknown')).toBe(0)
    })

    it('treats unrecognized levels as lowest', () => {
        expect(getThreatRank('bogus')).toBe(0)
        expect(getThreatRank('')).toBe(0)
    })
})

describe('getTagScore', () => {
    it('is zero with no flags', () => {
        expect(getTagScore(makePilot())).toBe(0)
    })

    it('weights super above everything else combined', () => {
        const superOnly = getTagScore(makePilot({ flags: { is_super: true } }))
        const allOthers = getTagScore(
            makePilot({
                flags: {
                    is_capital: true,
                    is_blops: true,
                    is_recon: true,
                    is_cyno: true,
                    is_solo: true,
                },
            })
        )
        expect(superOnly).toBe(100)
        expect(allOthers).toBe(96)
        expect(superOnly).toBeGreaterThan(allOthers)
    })

    it('sums individual flag weights', () => {
        expect(
            getTagScore(makePilot({ flags: { is_recon: true, is_solo: true } }))
        ).toBe(15)
    })
})

describe('getKdRatio', () => {
    it('is zero without zkill data', () => {
        expect(getKdRatio(makePilot())).toBe(0)
    })

    it('divides destroyed by lost', () => {
        const pilot = makePilot({
            zkill: makeZkill({ ships_destroyed: 30, ships_lost: 10 }),
        })
        expect(getKdRatio(pilot)).toBe(3)
    })

    it('returns raw kills when nothing was lost', () => {
        const pilot = makePilot({
            zkill: makeZkill({ ships_destroyed: 42, ships_lost: 0 }),
        })
        expect(getKdRatio(pilot)).toBe(42)
    })
})

describe('getPpk', () => {
    it('is zero without zkill data or kills', () => {
        expect(getPpk(makePilot())).toBe(0)
        expect(
            getPpk(makePilot({ zkill: makeZkill({ points_destroyed: 100 }) }))
        ).toBe(0)
    })

    it('divides points by kills', () => {
        const pilot = makePilot({
            zkill: makeZkill({ points_destroyed: 100, ships_destroyed: 20 }),
        })
        expect(getPpk(pilot)).toBe(5)
    })
})

describe('comparePilots', () => {
    it('compares corporation by name and corp by ticker', () => {
        const a = makePilot({
            corporation_name: 'Alpha Corp',
            corporation_ticker: 'ZZZ',
        })
        const b = makePilot({
            corporation_name: 'Zulu Corp',
            corporation_ticker: 'AAA',
        })
        expect(comparePilots(a, b, 'corporation')).toBeLessThan(0)
        expect(comparePilots(a, b, 'corp')).toBeGreaterThan(0)
    })

    it('compares alliance by name and ally by ticker', () => {
        const a = makePilot({
            alliance_name: 'Alpha Alliance',
            alliance_ticker: 'ZZZ',
        })
        const b = makePilot({
            alliance_name: 'Zulu Alliance',
            alliance_ticker: 'AAA',
        })
        expect(comparePilots(a, b, 'alliance')).toBeLessThan(0)
        expect(comparePilots(a, b, 'ally')).toBeGreaterThan(0)
    })

    it('treats missing names/tickers as empty strings', () => {
        const a = makePilot()
        const b = makePilot({ corporation_name: 'Some Corp' })
        expect(comparePilots(a, b, 'corporation')).toBeLessThan(0)
        expect(comparePilots(a, a, 'corporation')).toBe(0)
    })

    it('compares equal for unknown sort keys', () => {
        const a = makePilot({ threat: 'extreme' })
        const b = makePilot({ threat: 'minimal' })
        expect(comparePilots(a, b, 'nonsense')).toBe(0)
    })
})

describe('sortPilots', () => {
    it('sorts by threat descending by default direction', () => {
        const pilots = [
            makePilot({ name: 'Low', threat: 'low' }),
            makePilot({ name: 'Extreme', threat: 'extreme' }),
            makePilot({ name: 'Unknown', threat: 'unknown' }),
            makePilot({ name: 'High', threat: 'high' }),
        ]
        const sorted = sortPilots(pilots, 'threat', 'desc')
        expect(sorted.map((p) => p.character.name)).toEqual([
            'Extreme',
            'High',
            'Low',
            'Unknown',
        ])
    })

    it('inverts order for ascending', () => {
        const pilots = [
            makePilot({ name: 'Extreme', threat: 'extreme' }),
            makePilot({ name: 'Low', threat: 'low' }),
        ]
        const sorted = sortPilots(pilots, 'threat', 'asc')
        expect(sorted.map((p) => p.character.name)).toEqual(['Low', 'Extreme'])
    })

    it('sorts by pilot name', () => {
        const pilots = [
            makePilot({ name: 'Charlie' }),
            makePilot({ name: 'Alice' }),
            makePilot({ name: 'Bob' }),
        ]
        const sorted = sortPilots(pilots, 'pilot', 'asc')
        expect(sorted.map((p) => p.character.name)).toEqual([
            'Alice',
            'Bob',
            'Charlie',
        ])
    })

    it('sorts names case-insensitively', () => {
        const pilots = [
            makePilot({ name: 'charlie' }),
            makePilot({ name: 'ALICE' }),
            makePilot({ name: 'Bob' }),
        ]
        const sorted = sortPilots(pilots, 'pilot', 'asc')
        expect(sorted.map((p) => p.character.name)).toEqual([
            'ALICE',
            'Bob',
            'charlie',
        ])
    })

    it('keeps arrival order for names differing only by case', () => {
        const pilots = [
            makePilot({ name: 'alice' }),
            makePilot({ name: 'Alice' }),
        ]
        const sorted = sortPilots(pilots, 'pilot', 'asc')
        // sensitivity: 'base' compares them equal; the sort is stable.
        expect(sorted.map((p) => p.character.name)).toEqual(['alice', 'Alice'])
    })

    it('sorts by kd with infinite-kills pilots ranked by raw kills', () => {
        const pilots = [
            makePilot({
                name: 'Ratio3',
                zkill: makeZkill({ ships_destroyed: 30, ships_lost: 10 }),
            }),
            makePilot({
                name: 'NoLosses',
                zkill: makeZkill({ ships_destroyed: 50, ships_lost: 0 }),
            }),
            makePilot({ name: 'NoData' }),
        ]
        const sorted = sortPilots(pilots, 'kd', 'desc')
        expect(sorted.map((p) => p.character.name)).toEqual([
            'NoLosses',
            'Ratio3',
            'NoData',
        ])
    })

    it('does not mutate the input array', () => {
        const pilots = [
            makePilot({ name: 'B', threat: 'low' }),
            makePilot({ name: 'A', threat: 'extreme' }),
        ]
        const before = [...pilots]
        sortPilots(pilots, 'threat', 'desc')
        expect(pilots).toEqual(before)
    })
})
