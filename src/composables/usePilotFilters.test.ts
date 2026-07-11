import { describe, it, expect, vi } from 'vitest'
import { ref } from 'vue'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))
vi.mock('@tauri-apps/api/event', () => ({
    listen: vi.fn(() => Promise.resolve(() => {})),
}))
vi.mock('../utils/config', () => ({ API_BASE_URL: 'https://test.example.com' }))
vi.mock('../stores/intel', () => ({
    resolvePilotAnnotations: vi.fn(() => []),
    // Fresh identity per read so getPilotTags' memo cache never hits and
    // every call exercises the mocked resolvePilotAnnotations.
    annotationsByTargetKey: {
        get value() {
            return {}
        },
    },
}))

import { usePilotFilters } from './usePilotFilters'
import { resolvePilotAnnotations } from '../stores/intel'
import type { PilotIntel } from '../types'

function makePilot(
    overrides: Partial<PilotIntel> & { name?: string } = {}
): PilotIntel {
    const { name, ...rest } = overrides
    return {
        character: {
            id: (Math.random() * 100000) | 0,
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
        },
        error: null,
        ...rest,
    }
}

describe('usePilotFilters', () => {
    it('returns all pilots when no filters are active', () => {
        const pilots = ref([makePilot(), makePilot()])
        const { filteredPilots } = usePilotFilters(pilots)
        expect(filteredPilots.value).toHaveLength(2)
    })

    it('filters by threat level', () => {
        const pilots = ref([
            makePilot({ threat_level: 'extreme' }),
            makePilot({ threat_level: 'low' }),
        ])
        const { filteredPilots, toggleThreatFilter } = usePilotFilters(pilots)

        toggleThreatFilter('extreme')
        expect(filteredPilots.value).toHaveLength(1)
        expect(filteredPilots.value[0].threat_level).toBe('extreme')
    })

    it('filters by tag (zkill flag)', () => {
        const pilots = ref([
            makePilot({
                flags: {
                    is_cyno: true,
                    is_recon: false,
                    is_blops: false,
                    is_capital: false,
                    is_super: false,
                    is_solo: false,
                },
            }),
            makePilot(),
        ])
        const { filteredPilots, toggleTag } = usePilotFilters(pilots)

        toggleTag('CYNO')
        expect(filteredPilots.value).toHaveLength(1)
    })

    it('filters by corporation', () => {
        const p1 = makePilot()
        p1.character.corporation_name = 'Corp A'
        const p2 = makePilot()
        p2.character.corporation_name = 'Corp B'

        const pilots = ref([p1, p2])
        const { filteredPilots, toggleCorpFilter } = usePilotFilters(pilots)

        toggleCorpFilter('Corp A')
        expect(filteredPilots.value).toHaveLength(1)
        expect(filteredPilots.value[0].character.corporation_name).toBe(
            'Corp A'
        )
    })

    it('filters by alliance', () => {
        const p1 = makePilot()
        p1.character.alliance_name = 'Alliance A'
        const p2 = makePilot()
        p2.character.alliance_name = 'Alliance B'

        const pilots = ref([p1, p2])
        const { filteredPilots, toggleAllianceFilter } = usePilotFilters(pilots)

        toggleAllianceFilter('Alliance A')
        expect(filteredPilots.value).toHaveLength(1)
    })

    it('supports multi-select mode for tags', () => {
        const pilots = ref([
            makePilot({
                flags: {
                    is_cyno: true,
                    is_recon: false,
                    is_blops: false,
                    is_capital: false,
                    is_super: false,
                    is_solo: false,
                },
            }),
            makePilot({
                flags: {
                    is_cyno: false,
                    is_recon: true,
                    is_blops: false,
                    is_capital: false,
                    is_super: false,
                    is_solo: false,
                },
            }),
            makePilot(),
        ])
        const { filteredPilots, toggleTag } = usePilotFilters(pilots)

        toggleTag('CYNO')
        toggleTag('RECON')
        expect(filteredPilots.value).toHaveLength(2)
    })

    it('clearFilters resets everything', () => {
        const pilots = ref([
            makePilot({ threat_level: 'extreme' }),
            makePilot(),
        ])
        const { filteredPilots, toggleThreatFilter, toggleTag, clearFilters } =
            usePilotFilters(pilots)

        toggleThreatFilter('extreme')
        toggleTag('CYNO')
        expect(filteredPilots.value.length).toBeLessThan(2)

        clearFilters()
        expect(filteredPilots.value).toHaveLength(2)
    })

    it('filters by intel annotation tags', () => {
        const pilot = makePilot()
        const pilots = ref([pilot, makePilot()])

        vi.mocked(resolvePilotAnnotations).mockImplementation((p) => {
            if (p.character.id === pilot.character.id) {
                return [
                    {
                        key: 'character:1',
                        scope: 'character' as const,
                        annotation: {
                            id: 1,
                            networkId: 1,
                            networkName: 'Net',
                            targetType: 'character' as const,
                            targetId: pilot.character.id,
                            targetName: pilot.character.name,
                            tags: ['HOSTILE'],
                            note: null,
                            color: '#FF3B3B',
                            createdBy: null,
                        },
                    },
                ]
            }
            return []
        })

        const { filteredPilots, toggleTag } = usePilotFilters(pilots)
        toggleTag('HOSTILE')
        expect(filteredPilots.value).toHaveLength(1)
        expect(filteredPilots.value[0].character.id).toBe(pilot.character.id)
    })
})
