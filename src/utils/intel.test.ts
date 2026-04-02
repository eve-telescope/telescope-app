import { describe, it, expect } from 'vitest'
import { getFlagLabels } from './intel'
import type { PilotFlags } from '../types'

function makeFlags(overrides: Partial<PilotFlags> = {}): PilotFlags {
    return {
        is_cyno: false,
        is_recon: false,
        is_blops: false,
        is_capital: false,
        is_super: false,
        is_solo: false,
        ...overrides,
    }
}

describe('getFlagLabels', () => {
    it('returns empty for no flags', () => {
        expect(getFlagLabels(makeFlags())).toEqual([])
    })

    it('returns SUPER (excludes CAPITAL)', () => {
        expect(
            getFlagLabels(makeFlags({ is_super: true, is_capital: true }))
        ).toContain('SUPER')
        expect(
            getFlagLabels(makeFlags({ is_super: true, is_capital: true }))
        ).not.toContain('CAPITAL')
    })

    it('returns CAPITAL when not SUPER', () => {
        expect(getFlagLabels(makeFlags({ is_capital: true }))).toContain(
            'CAPITAL'
        )
    })

    it('returns multiple flags', () => {
        const labels = getFlagLabels(
            makeFlags({ is_cyno: true, is_recon: true, is_solo: true })
        )
        expect(labels).toContain('CYNO')
        expect(labels).toContain('RECON')
        expect(labels).toContain('SOLO')
    })

    it('returns BLACK OPS', () => {
        expect(getFlagLabels(makeFlags({ is_blops: true }))).toContain(
            'BLACK OPS'
        )
    })
})
