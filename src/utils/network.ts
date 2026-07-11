import type { EntityType, NetworkAccess, PermissionLevel } from '../types'
import { normalizeAnnotationTag, parseAnnotationTags } from './annotations'

const TAG_JOIN = ' | '

/** Entity selected in a network dialog form (EntityCombobox model shape). */
export interface NetworkEntitySelection {
    id: string
    name: string
    type: EntityType | null
}

/** Payload emitted by AnnotationDialog when the user saves the form. */
export interface AnnotationSavePayload {
    editingId: number | null
    entity: { id: string; name: string; type: EntityType }
    tags: string[]
    note: string | null
}

/** Payload emitted by NetworkAccessDialog when the user grants access. */
export interface AccessSavePayload {
    entity: { id: string; name: string; type: EntityType }
    permission: PermissionLevel
}

/** EVE image-server portrait/logo URL for a character, corporation, or alliance. */
export function getPortraitUrl(type: string, id: number, size = 32): string {
    if (type === 'character' || type.includes('User')) {
        return `https://images.evetech.net/characters/${id}/portrait?size=${size}`
    }
    if (type === 'corporation' || type.includes('Corporation')) {
        return `https://images.evetech.net/corporations/${id}/logo?size=${size}`
    }
    if (type === 'alliance' || type.includes('Alliance')) {
        return `https://images.evetech.net/alliances/${id}/logo?size=${size}`
    }
    return ''
}

/** Owner accesses can never be revoked from the access list. */
export function canRemoveAccess(
    access: Pick<NetworkAccess, 'is_owner'>
): boolean {
    return !access.is_owner
}

/** Badge label for an access row: 'owner' trumps the permission level. */
export function accessPermissionLabel(
    access: Pick<NetworkAccess, 'is_owner' | 'permission'>
): string {
    return access.is_owner ? 'owner' : access.permission
}

export const PERMISSION_DESCRIPTIONS: Record<PermissionLevel, string> = {
    viewer: 'Can view intel entries',
    member: 'Can view and add/edit entries',
    manager: 'Full control including access management',
}

export function describePermission(level: PermissionLevel): string {
    return PERMISSION_DESCRIPTIONS[level]
}

/**
 * Toggle a preset tag inside a raw `|`-separated tags input string,
 * preserving the order of the remaining tags.
 */
export function toggleAnnotationTag(tagsInput: string, tag: string): string {
    const tags = parseAnnotationTags(tagsInput)
    const normalized = normalizeAnnotationTag(tag)
    const idx = tags.indexOf(normalized)
    if (idx >= 0) {
        tags.splice(idx, 1)
    } else {
        tags.push(normalized)
    }
    return tags.join(TAG_JOIN)
}

/** An annotation form with no tags and no note carries no information. */
export function isAnnotationFormEmpty(
    tags: string[],
    note: string | null
): boolean {
    return tags.length === 0 && !note
}

export type AnnotationSaveAction = 'create' | 'update' | 'delete' | 'none'

/**
 * Decide what saving the annotation form means:
 * - editing an entry and clearing it out deletes the entry
 * - editing with content updates it
 * - a fresh form with content creates a new entry
 * - a fresh, empty form is a no-op (dialog simply closes)
 */
export function resolveAnnotationSaveAction(
    editingId: number | null,
    isEmpty: boolean
): AnnotationSaveAction {
    if (editingId != null) {
        return isEmpty ? 'delete' : 'update'
    }
    return isEmpty ? 'none' : 'create'
}
