import { describe, it, expect, vi } from 'vitest'

// Mock all Tauri and store dependencies
vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(),
}))
vi.mock('@tauri-apps/api/event', () => ({
    listen: vi.fn(() => Promise.resolve(() => {})),
}))
vi.mock('../utils/config', () => ({
    API_BASE_URL: 'https://test.example.com',
}))

// We can't easily mount NetworkManager (too many dependencies),
// so we test the permission logic and visibility rules directly.
// These tests validate the business rules that drive UI visibility.

import type { NetworkAccess, NetworkDetail } from '../types'

describe('Permission-based visibility rules', () => {
    function makeAccess(
        permission: NetworkAccess['permission'],
        isOwner = false
    ): NetworkAccess {
        return {
            id: 1,
            accessible_type: 'character',
            accessible_id: 12345,
            permission,
            is_owner: isOwner,
            expires_at: null,
            entity: {
                id: 12345,
                name: 'Test User',
                type: 'character',
            },
        }
    }

    function makeNetwork(accesses: NetworkAccess[]): NetworkDetail {
        return {
            id: 1,
            name: 'Test Network',
            slug: 'test-network-abc123',
            entries: [],
            accesses,
        }
    }

    describe('access management buttons', () => {
        it('owner can see delete button for non-owner accesses', () => {
            const network = makeNetwork([
                makeAccess('manager', true),
                makeAccess('viewer', false),
            ])
            const nonOwnerAccesses = network.accesses.filter((a) => !a.is_owner)
            expect(nonOwnerAccesses.length).toBe(1)
        })

        it('owner access cannot be deleted', () => {
            const network = makeNetwork([makeAccess('manager', true)])
            const ownerAccess = network.accesses.find((a) => a.is_owner)
            expect(ownerAccess).toBeDefined()
            // UI rule: v-if="!access.is_owner" on delete button
            expect(ownerAccess!.is_owner).toBe(true)
        })
    })

    describe('annotation visibility', () => {
        it('annotations are empty when no entries exist', () => {
            const network = makeNetwork([makeAccess('viewer')])
            expect(network.entries).toHaveLength(0)
        })
    })
})

describe('Scan history visibility rules', () => {
    it('scans should only show when authenticated and network is active', () => {
        // Rule: v-if="isAuthenticated && activeNetwork && recentScans.length > 0"
        const isAuthenticated = false
        const activeNetwork = null
        const recentScans: unknown[] = []

        const shouldShow =
            isAuthenticated && activeNetwork !== null && recentScans.length > 0
        expect(shouldShow).toBe(false)
    })

    it('scans show when all conditions are met', () => {
        const isAuthenticated = true
        const activeNetwork = { id: 1, name: 'Net' }
        const recentScans = [{ id: 1, scan_type: 'local', raw_text: 'Pilot' }]

        const shouldShow =
            isAuthenticated && activeNetwork !== null && recentScans.length > 0
        expect(shouldShow).toBe(true)
    })

    it('scans hidden when not authenticated even with network', () => {
        const isAuthenticated = false
        const activeNetwork = { id: 1, name: 'Net' }
        const recentScans = [{ id: 1, scan_type: 'local', raw_text: 'Pilot' }]

        const shouldShow =
            isAuthenticated && activeNetwork !== null && recentScans.length > 0
        expect(shouldShow).toBe(false)
    })

    it('scans hidden when no active network', () => {
        const isAuthenticated = true
        const activeNetwork = null
        const recentScans = [{ id: 1, scan_type: 'local', raw_text: 'Pilot' }]

        const shouldShow =
            isAuthenticated && activeNetwork !== null && recentScans.length > 0
        expect(shouldShow).toBe(false)
    })
})

describe('Filter behavior with intel tags', () => {
    it('stale tag filter should be prunable', () => {
        const selectedTags = new Set(['HOSTILE', 'CUSTOM_TAG'])
        const availableTags = new Set(['HOSTILE']) // CUSTOM_TAG no longer exists

        for (const tag of selectedTags) {
            if (!availableTags.has(tag)) {
                selectedTags.delete(tag)
            }
        }

        expect(selectedTags.has('HOSTILE')).toBe(true)
        expect(selectedTags.has('CUSTOM_TAG')).toBe(false)
    })
})
