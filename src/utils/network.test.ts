import { describe, it, expect } from 'vitest'
import {
    accessPermissionLabel,
    canRemoveAccess,
    describePermission,
    getPortraitUrl,
    isAnnotationFormEmpty,
    resolveAnnotationSaveAction,
    toggleAnnotationTag,
} from './network'

describe('getPortraitUrl', () => {
    it('builds character portrait URLs', () => {
        expect(getPortraitUrl('character', 42)).toBe(
            'https://images.evetech.net/characters/42/portrait?size=32'
        )
    })

    it('builds corporation logo URLs', () => {
        expect(getPortraitUrl('corporation', 7)).toBe(
            'https://images.evetech.net/corporations/7/logo?size=32'
        )
    })

    it('builds alliance logo URLs with a custom size', () => {
        expect(getPortraitUrl('alliance', 9, 64)).toBe(
            'https://images.evetech.net/alliances/9/logo?size=64'
        )
    })

    it('matches backend-style type names', () => {
        expect(getPortraitUrl('App\\Models\\User', 1)).toContain('/characters/')
        expect(getPortraitUrl('App\\Models\\Corporation', 1)).toContain(
            '/corporations/'
        )
        expect(getPortraitUrl('App\\Models\\Alliance', 1)).toContain(
            '/alliances/'
        )
    })

    it('returns an empty string for unknown types', () => {
        expect(getPortraitUrl('unknown', 1)).toBe('')
    })
})

describe('canRemoveAccess', () => {
    it('allows removing non-owner accesses', () => {
        expect(canRemoveAccess({ is_owner: false })).toBe(true)
    })

    it('never allows removing the owner access', () => {
        expect(canRemoveAccess({ is_owner: true })).toBe(false)
    })
})

describe('accessPermissionLabel', () => {
    it('labels the owner as owner regardless of permission', () => {
        expect(
            accessPermissionLabel({ is_owner: true, permission: 'manager' })
        ).toBe('owner')
    })

    it('labels non-owners with their permission level', () => {
        expect(
            accessPermissionLabel({ is_owner: false, permission: 'viewer' })
        ).toBe('viewer')
        expect(
            accessPermissionLabel({ is_owner: false, permission: 'member' })
        ).toBe('member')
    })
})

describe('describePermission', () => {
    it('describes each permission level', () => {
        expect(describePermission('viewer')).toBe('Can view intel entries')
        expect(describePermission('member')).toBe(
            'Can view and add/edit entries'
        )
        expect(describePermission('manager')).toBe(
            'Full control including access management'
        )
    })
})

describe('toggleAnnotationTag', () => {
    it('adds a tag to an empty input', () => {
        expect(toggleAnnotationTag('', 'HOSTILE')).toBe('HOSTILE')
    })

    it('appends a tag to existing tags with the standard delimiter', () => {
        expect(toggleAnnotationTag('HOSTILE', 'SCOUT')).toBe('HOSTILE | SCOUT')
    })

    it('removes a tag that is already present', () => {
        expect(toggleAnnotationTag('HOSTILE | SCOUT', 'HOSTILE')).toBe('SCOUT')
    })

    it('normalizes the toggled tag before comparing', () => {
        expect(toggleAnnotationTag('HOSTILE', '  hostile ')).toBe('')
    })

    it('preserves the order of remaining tags', () => {
        expect(toggleAnnotationTag('a | b | c', 'B')).toBe('A | C')
    })
})

describe('isAnnotationFormEmpty', () => {
    it('is empty with no tags and no note', () => {
        expect(isAnnotationFormEmpty([], null)).toBe(true)
    })

    it('is not empty with tags', () => {
        expect(isAnnotationFormEmpty(['HOSTILE'], null)).toBe(false)
    })

    it('is not empty with a note', () => {
        expect(isAnnotationFormEmpty([], 'seen in Jita')).toBe(false)
    })
})

describe('resolveAnnotationSaveAction', () => {
    it('deletes when editing an entry and the form was cleared', () => {
        expect(resolveAnnotationSaveAction(5, true)).toBe('delete')
    })

    it('updates when editing an entry with content', () => {
        expect(resolveAnnotationSaveAction(5, false)).toBe('update')
    })

    it('creates when adding a new entry with content', () => {
        expect(resolveAnnotationSaveAction(null, false)).toBe('create')
    })

    it('does nothing for a fresh empty form', () => {
        expect(resolveAnnotationSaveAction(null, true)).toBe('none')
    })
})
