import { describe, it, expect, vi } from 'vitest'

// Mock the intel store before importing pilotTags
vi.mock('../stores/intel', () => ({
    resolvePilotAnnotations: vi.fn(() => []),
}))

import { getPilotTags, getPilotTagStrings } from './pilotTags'
import { resolvePilotAnnotations } from '../stores/intel'
import type { PilotIntel } from '../types'

function makePilot(overrides: Partial<PilotIntel['flags']> = {}): PilotIntel {
    return {
        character: {
            id: 1,
            name: 'Test Pilot',
            corporation_id: null,
            corporation_name: null,
            corporation_ticker: null,
            alliance_id: null,
            alliance_name: null,
            alliance_ticker: null,
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
            ...overrides,
        },
        error: null,
    }
}

describe('getPilotTags', () => {
    it('returns empty for pilot with no flags and no annotations', () => {
        expect(getPilotTags(makePilot())).toEqual([])
    })

    it('returns zkill flag tags', () => {
        const tags = getPilotTags(makePilot({ is_cyno: true, is_recon: true }))
        const tagNames = tags.map((t) => t.tag)
        expect(tagNames).toContain('CYNO')
        expect(tagNames).toContain('RECON')
    })

    it('includes SUPER flag', () => {
        const tags = getPilotTags(makePilot({ is_super: true }))
        expect(tags.map((t) => t.tag)).toContain('SUPER')
    })

    it('includes annotation tags', () => {
        vi.mocked(resolvePilotAnnotations).mockReturnValueOnce([
            {
                key: 'character:1',
                scope: 'character' as const,
                annotation: {
                    id: 1,
                    networkId: 1,
                    networkName: 'Test',
                    targetType: 'character' as const,
                    targetId: 1,
                    targetName: 'Test Pilot',
                    tags: ['HOSTILE', 'SPY'],
                    note: null,
                    color: '#FF3B3B',
                    createdBy: null,
                },
            },
        ])

        const tags = getPilotTags(makePilot())
        const tagNames = tags.map((t) => t.tag)
        expect(tagNames).toContain('HOSTILE')
        expect(tagNames).toContain('SPY')
    })

    it('deduplicates tags from flags and annotations', () => {
        vi.mocked(resolvePilotAnnotations).mockReturnValueOnce([
            {
                key: 'character:1',
                scope: 'character' as const,
                annotation: {
                    id: 1,
                    networkId: 1,
                    networkName: 'Test',
                    targetType: 'character' as const,
                    targetId: 1,
                    targetName: 'Test Pilot',
                    tags: ['CYNO'],
                    note: null,
                    color: '#A855F7',
                    createdBy: null,
                },
            },
        ])

        const tags = getPilotTags(makePilot({ is_cyno: true }))
        const cynoTags = tags.filter((t) => t.tag === 'CYNO')
        expect(cynoTags).toHaveLength(1)
    })
})

describe('getPilotTagStrings', () => {
    it('returns tag name strings', () => {
        const strings = getPilotTagStrings(
            makePilot({ is_capital: true, is_solo: true })
        )
        expect(strings).toContain('CAPITAL')
        expect(strings).toContain('SOLO')
    })
})
