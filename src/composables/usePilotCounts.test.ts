import { describe, it, expect, vi } from 'vitest'
import { ref } from 'vue'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))
vi.mock('@tauri-apps/api/event', () => ({
    listen: vi.fn(() => Promise.resolve(() => {})),
}))
vi.mock('../utils/config', () => ({ API_BASE_URL: 'https://test.example.com' }))
vi.mock('../stores/intel', () => ({
    resolvePilotAnnotations: vi.fn(() => []),
}))

import { usePilotCounts } from './usePilotCounts'
import { resolvePilotAnnotations } from '../stores/intel'
import type { PilotIntel } from '../types'

function makePilot(
    overrides: {
        threat?: string
        flags?: Partial<PilotIntel['flags']>
        corp?: string
        alliance?: string
    } = {}
): PilotIntel {
    return {
        character: {
            id: (Math.random() * 100000) | 0,
            name: 'Pilot',
            corporation_id: 98000001,
            corporation_name: overrides.corp ?? 'Test Corp',
            corporation_ticker: 'TSTC',
            alliance_id: 99000001,
            alliance_name: overrides.alliance ?? 'Test Alliance',
            alliance_ticker: 'TSTA',
        },
        zkill: null,
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

describe('usePilotCounts', () => {
    describe('threatCounts', () => {
        it('counts pilots by threat level', () => {
            const pilots = ref([
                makePilot({ threat: 'extreme' }),
                makePilot({ threat: 'extreme' }),
                makePilot({ threat: 'low' }),
            ])
            const { threatCounts } = usePilotCounts(pilots)
            expect(threatCounts.value.extreme).toBe(2)
            expect(threatCounts.value.low).toBe(1)
            expect(threatCounts.value.high).toBe(0)
        })
    })

    describe('tagCounts', () => {
        it('counts zkill flag tags', () => {
            const pilots = ref([
                makePilot({ flags: { is_cyno: true } }),
                makePilot({ flags: { is_cyno: true, is_recon: true } }),
                makePilot(),
            ])
            const { tagCounts } = usePilotCounts(pilots)
            const cyno = tagCounts.value.find((t) => t.tag === 'CYNO')
            const recon = tagCounts.value.find((t) => t.tag === 'RECON')
            expect(cyno?.count).toBe(2)
            expect(recon?.count).toBe(1)
        })

        it('includes intel annotation tags', () => {
            const pilot = makePilot()
            const pilots = ref([pilot])

            vi.mocked(resolvePilotAnnotations).mockReturnValue([
                {
                    key: 'char:1',
                    scope: 'character' as const,
                    annotation: {
                        id: 1,
                        networkId: 1,
                        networkName: 'Net',
                        targetType: 'character' as const,
                        targetId: pilot.character.id,
                        targetName: 'Pilot',
                        tags: ['HOSTILE'],
                        note: null,
                        color: '#FF3B3B',
                        createdBy: null,
                    },
                },
            ])

            const { tagCounts } = usePilotCounts(pilots)
            const hostile = tagCounts.value.find((t) => t.tag === 'HOSTILE')
            expect(hostile).toBeDefined()
            expect(hostile!.count).toBe(1)
        })

        it('counts each pilot only once per tag', () => {
            const pilot = makePilot({ flags: { is_cyno: true } })
            const pilots = ref([pilot])

            // Annotation also has CYNO — should still count as 1
            vi.mocked(resolvePilotAnnotations).mockReturnValue([
                {
                    key: 'char:1',
                    scope: 'character' as const,
                    annotation: {
                        id: 1,
                        networkId: 1,
                        networkName: 'Net',
                        targetType: 'character' as const,
                        targetId: pilot.character.id,
                        targetName: 'Pilot',
                        tags: ['CYNO'],
                        note: null,
                        color: '#A855F7',
                        createdBy: null,
                    },
                },
            ])

            const { tagCounts } = usePilotCounts(pilots)
            const cyno = tagCounts.value.find((t) => t.tag === 'CYNO')
            expect(cyno?.count).toBe(1)
        })
    })

    describe('corpCounts', () => {
        it('counts pilots by corporation ticker', () => {
            const p1 = makePilot({ corp: 'Corp A' })
            p1.character.corporation_ticker = 'CRPA'
            const p2 = makePilot({ corp: 'Corp A' })
            p2.character.corporation_ticker = 'CRPA'
            const p3 = makePilot({ corp: 'Corp B' })
            p3.character.corporation_ticker = 'CRPB'

            const pilots = ref([p1, p2, p3])
            const { corpCounts } = usePilotCounts(pilots)
            expect(corpCounts.value).toHaveLength(2)
            expect(corpCounts.value[0].count).toBe(2)
        })
    })

    describe('allianceCounts', () => {
        it('counts pilots by alliance ticker', () => {
            const p1 = makePilot({ alliance: 'Alliance X' })
            p1.character.alliance_ticker = 'ALLX'
            const p2 = makePilot({ alliance: 'Alliance X' })
            p2.character.alliance_ticker = 'ALLX'
            const p3 = makePilot({ alliance: 'Alliance Y' })
            p3.character.alliance_ticker = 'ALLY'

            const pilots = ref([p1, p2, p3])
            const { allianceCounts } = usePilotCounts(pilots)
            expect(allianceCounts.value).toHaveLength(2)
        })
    })
})
