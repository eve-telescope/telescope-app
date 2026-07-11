import { describe, it, expect, vi } from 'vitest'

// Mock Tauri APIs before importing the store
vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn((cmd: string) => {
        if (cmd === 'get_intel_state') {
            return Promise.resolve({
                api_base_url: 'https://test.example.com',
                api_token: null,
                networks: [],
                entries: [],
                selected_network: null,
                active_network_ids: [],
            })
        }
        return Promise.resolve(null)
    }),
}))
vi.mock('@tauri-apps/api/event', () => ({
    listen: vi.fn(() => Promise.resolve(() => {})),
}))
vi.mock('../utils/config', () => ({
    API_BASE_URL: 'https://test.example.com',
}))

import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import {
    initIntelStore,
    isAuthenticated,
    entries,
    activeNetworkId,
    annotations,
    annotationsByTargetKey,
    getAnnotationsForPilot,
    resolvePilotAnnotations,
    onEntryCreated,
    onEntryUpdated,
    onEntryDeleted,
    onScanShared,
    latestSharedScan,
} from './intel'
import type { IntelEntry, NetworkScan, PilotIntel } from '../types'

// Helper to set internal state by emitting a state-changed event
// Since the store uses module-level refs, we can write to them via the exported computeds' source
// The store listens for 'intel-state-changed' events, but since we mocked listen,
// we need to manipulate exports directly via the event handlers.

// Actually, we can't set state directly since it's a private ref.
// But we CAN test the event handlers and computed derivations.

function makeEntry(overrides: Partial<IntelEntry> = {}): IntelEntry {
    return {
        id: 1,
        intel_network_id: 1,
        network_name: 'Test Network',
        entity_type: 'character',
        entity_id: 12345,
        entity_name: 'Bad Guy',
        color: '#FF3B3B',
        label: 'HOSTILE',
        notes: null,
        ...overrides,
    }
}

function makeScan(overrides: Partial<NetworkScan> = {}): NetworkScan {
    return {
        id: 1,
        scan_type: 'local',
        raw_text: 'Pilot One\nPilot Two',
        solar_system: null,
        created_at: '2026-04-01T10:00:00+00:00',
        submitted_by: { id: 12345, character_name: 'Scanner' },
        ...overrides,
    }
}

function makePilot(overrides: Partial<PilotIntel> = {}): PilotIntel {
    return {
        character: {
            id: 12345,
            name: 'Test Pilot',
            corporation_id: 98000001,
            corporation_name: 'Test Corp',
            corporation_ticker: 'TSTC',
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
        },
        error: null,
        ...overrides,
    }
}

// ---------------------------------------------------------------------------
// Echo event handlers
// ---------------------------------------------------------------------------

describe('onEntryCreated', () => {
    it('adds a new entry to the entries list', () => {
        const entry = makeEntry({ id: 9000 })
        onEntryCreated(entry)
        expect(entries.value.find((e) => e.id === 9000)).toBeDefined()
    })

    it('updates an existing entry with the same ID', () => {
        const entry = makeEntry({ id: 9001, label: 'SPY' })
        onEntryCreated(entry)
        const updated = makeEntry({ id: 9001, label: 'FRIENDLY' })
        onEntryCreated(updated)

        const found = entries.value.filter((e) => e.id === 9001)
        expect(found).toHaveLength(1)
        expect(found[0].label).toBe('FRIENDLY')
    })
})

describe('onEntryUpdated', () => {
    it('updates the entry in place', () => {
        const entry = makeEntry({ id: 9002, label: 'HOSTILE' })
        onEntryCreated(entry)

        onEntryUpdated(makeEntry({ id: 9002, label: 'NEUTRAL' }))
        expect(entries.value.find((e) => e.id === 9002)?.label).toBe('NEUTRAL')
    })
})

describe('onEntryDeleted', () => {
    it('removes the entry', () => {
        const entry = makeEntry({ id: 9003 })
        onEntryCreated(entry)
        expect(entries.value.find((e) => e.id === 9003)).toBeDefined()

        onEntryDeleted(9003)
        expect(entries.value.find((e) => e.id === 9003)).toBeUndefined()
    })
})

// ---------------------------------------------------------------------------
// Annotations computed
// ---------------------------------------------------------------------------

describe('annotations computed', () => {
    it('maps entries to annotations', () => {
        onEntryCreated(
            makeEntry({ id: 9010, label: 'HOSTILE | SPY', color: '#FF3B3B' })
        )
        const annotation = annotations.value.find((a) => a.id === 9010)
        expect(annotation).toBeDefined()
        expect(annotation!.tags).toContain('HOSTILE')
        expect(annotation!.tags).toContain('SPY')
    })

    it('handles null label gracefully', () => {
        onEntryCreated(
            makeEntry({ id: 9011, label: null, notes: 'Just a note' })
        )
        const annotation = annotations.value.find((a) => a.id === 9011)
        expect(annotation).toBeDefined()
        expect(annotation!.tags).toEqual([])
        expect(annotation!.note).toBe('Just a note')
    })
})

describe('annotationsByTargetKey', () => {
    it('indexes annotations by type:id', () => {
        onEntryCreated(
            makeEntry({
                id: 9020,
                entity_type: 'corporation',
                entity_id: 98000001,
                label: 'HOSTILE',
            })
        )
        const key = 'corporation:98000001'
        expect(annotationsByTargetKey.value[key]).toBeDefined()
        expect(annotationsByTargetKey.value[key].length).toBeGreaterThanOrEqual(
            1
        )
    })
})

describe('getAnnotationsForPilot', () => {
    it('returns annotations matching character, corp, or alliance ID', () => {
        onEntryCreated(
            makeEntry({
                id: 9030,
                entity_type: 'character',
                entity_id: 55555,
                label: 'SPY',
            })
        )
        onEntryCreated(
            makeEntry({
                id: 9031,
                entity_type: 'corporation',
                entity_id: 98000001,
                label: 'HOSTILE',
            })
        )

        const results = getAnnotationsForPilot(55555, 98000001, null)
        expect(results.length).toBeGreaterThanOrEqual(2)
        expect(results.some((r) => r.scope === 'character')).toBe(true)
        expect(results.some((r) => r.scope === 'corporation')).toBe(true)
    })

    it('returns empty when no annotations match', () => {
        const results = getAnnotationsForPilot(99999, 99998, 99997)
        expect(results).toEqual([])
    })
})

describe('resolvePilotAnnotations', () => {
    it('resolves annotations for a pilot object', () => {
        onEntryCreated(
            makeEntry({
                id: 9040,
                entity_type: 'character',
                entity_id: 12345,
                label: 'HOSTILE',
            })
        )

        const pilot = makePilot()
        const results = resolvePilotAnnotations(pilot)
        expect(results.some((r) => r.annotation.tags.includes('HOSTILE'))).toBe(
            true
        )
    })
})

// ---------------------------------------------------------------------------
// Scan sharing
// ---------------------------------------------------------------------------

describe('onScanShared', () => {
    it('updates latestSharedScan ref', () => {
        const scan = makeScan({ id: 100 })
        onScanShared(scan)
        expect(latestSharedScan.value).toBeDefined()
        expect(latestSharedScan.value!.id).toBe(100)
    })

    it('overwrites previous scan', () => {
        onScanShared(makeScan({ id: 101 }))
        onScanShared(makeScan({ id: 102 }))
        expect(latestSharedScan.value!.id).toBe(102)
    })
})

// ---------------------------------------------------------------------------
// Computed state derivations
// ---------------------------------------------------------------------------

describe('isAuthenticated', () => {
    // This depends on the internal state which is set via Rust events.
    // Since we can't easily set api_token, we test the computed logic indirectly.
    // The default state has api_token: null, so isAuthenticated should be false.
    it('is false by default', () => {
        // The store initializes with api_token: null
        expect(isAuthenticated.value).toBe(false)
    })
})

describe('activeNetworkId', () => {
    it('is null by default', () => {
        expect(activeNetworkId.value).toBeNull()
    })
})

// ---------------------------------------------------------------------------
// Bootstrap (kept last: initIntelStore replaces the state with the mocked
// get_intel_state snapshot, which would clear entries built by tests above)
// ---------------------------------------------------------------------------

describe('initIntelStore', () => {
    it('does not bootstrap at module import time', () => {
        // The store module was imported at the top of this file, but no
        // Tauri call may happen until initIntelStore() is invoked.
        expect(invoke).not.toHaveBeenCalled()
        expect(listen).not.toHaveBeenCalled()
    })

    it('subscribes and bootstraps exactly once, even when called twice', async () => {
        const first = initIntelStore()
        const second = initIntelStore()
        expect(second).toBe(first)
        await first

        expect(listen).toHaveBeenCalledTimes(1)
        expect(vi.mocked(listen).mock.calls[0][0]).toBe('intel-state-changed')
        expect(invoke).toHaveBeenCalledWith('set_api_base_url', {
            url: 'https://test.example.com',
        })
        expect(
            vi
                .mocked(invoke)
                .mock.calls.filter(([cmd]) => cmd === 'set_api_base_url')
        ).toHaveLength(1)

        // Not authenticated in the mocked snapshot → no network fetch.
        expect(invoke).not.toHaveBeenCalledWith('fetch_networks')

        // A later call is a no-op returning the same settled promise.
        await initIntelStore()
        expect(listen).toHaveBeenCalledTimes(1)
    })
})
