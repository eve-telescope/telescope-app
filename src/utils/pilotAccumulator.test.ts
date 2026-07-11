import { describe, it, expect } from 'vitest'
import { PilotAccumulator } from './pilotAccumulator'
import type { PilotIntel } from '../types'

function makePilot(id: number, threat: string, name?: string): PilotIntel {
    return {
        character: {
            id,
            name: name ?? `Pilot ${id}`,
            corporation_id: null,
            corporation_name: null,
            corporation_ticker: null,
            alliance_id: null,
            alliance_name: null,
            alliance_ticker: null,
        },
        zkill: null,
        threat_level: threat,
        flags: {
            is_cyno: false,
            is_recon: false,
            is_blops: false,
            is_capital: false,
            is_super: false,
            is_solo: false,
        },
        error: null,
    }
}

describe('PilotAccumulator', () => {
    it('adds pilots and reports size', () => {
        const acc = new PilotAccumulator()
        acc.upsert(makePilot(1, 'HIGH'))
        acc.upsert(makePilot(2, 'LOW'))
        expect(acc.size).toBe(2)
    })

    it('replaces the entry for an existing character id', () => {
        const acc = new PilotAccumulator()
        acc.upsert(makePilot(1, 'LOW', 'Stale'))
        acc.upsert(makePilot(1, 'HIGH', 'Fresh'))
        expect(acc.size).toBe(1)
        expect(acc.toArray()[0].character.name).toBe('Fresh')
        expect(acc.toArray()[0].threat_level).toBe('HIGH')
    })

    it('retainWhere keeps only matching pilots', () => {
        const acc = new PilotAccumulator()
        acc.upsert(makePilot(1, 'HIGH', 'Keep Me'))
        acc.upsert(makePilot(2, 'LOW', 'Drop Me'))
        acc.upsert(makePilot(3, 'EXTREME', 'Also Keep'))

        const wanted = new Set(['keep me', 'also keep'])
        acc.retainWhere((p) => wanted.has(p.character.name.toLowerCase()))

        expect(acc.size).toBe(2)
        expect(acc.toArray().map((p) => p.character.name)).toEqual([
            'Keep Me',
            'Also Keep',
        ])
    })

    it('returns pilots in insertion order regardless of threat', () => {
        const acc = new PilotAccumulator()
        acc.upsert(makePilot(1, 'Unknown'))
        acc.upsert(makePilot(2, 'LOW'))
        acc.upsert(makePilot(3, 'EXTREME'))
        acc.upsert(makePilot(4, 'MODERATE'))

        expect(acc.toArray().map((p) => p.threat_level)).toEqual([
            'Unknown',
            'LOW',
            'EXTREME',
            'MODERATE',
        ])
    })

    it('upserting an existing pilot keeps its original position', () => {
        const acc = new PilotAccumulator()
        acc.upsert(makePilot(1, 'HIGH', 'First'))
        acc.upsert(makePilot(2, 'EXTREME', 'Second'))
        acc.upsert(makePilot(1, 'LOW', 'First Updated'))

        const names = acc.toArray().map((p) => p.character.name)
        expect(names).toEqual(['First Updated', 'Second'])
    })

    it('clear empties the accumulator', () => {
        const acc = new PilotAccumulator()
        acc.upsert(makePilot(1, 'HIGH'))
        acc.clear()
        expect(acc.size).toBe(0)
        expect(acc.toArray()).toEqual([])
        // The same pilot can be added again after clearing
        acc.upsert(makePilot(1, 'HIGH'))
        expect(acc.size).toBe(1)
    })
})
