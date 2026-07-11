import type { PilotIntel } from '../types'
import { getPilotTagStrings } from './pilotTags'

/**
 * Shared filtering core for pilot lists. Both the main window
 * (usePilots/useScanFilters, usePilotFilters) and the overlay
 * (useSyncedFilters) build a PilotFilterState from their own reactive
 * state and delegate the actual matching here, so the semantics can't
 * drift between windows again.
 *
 * Fields left undefined are treated as inactive. Categories AND together;
 * multi-select sets OR within their category.
 */
export interface PilotFilterState {
    /** Lowercased threat level ('extreme', 'high', ...). */
    threatFilter?: string | null
    /** Tag strings as produced by getPilotTagStrings (flags + intel tags). */
    selectedTags?: ReadonlySet<string>
    /** Single-select corporation ticker. */
    corpFilter?: string | null
    /** Single-select alliance ticker. */
    allianceFilter?: string | null
    /** Multi-select corporation names ('Unknown' matches missing corp). */
    selectedCorps?: ReadonlySet<string>
    /** Multi-select alliance names (pilots without an alliance never match). */
    selectedAlliances?: ReadonlySet<string>
}

export function hasActiveFilters(state: PilotFilterState): boolean {
    return (
        state.threatFilter != null ||
        (state.selectedTags?.size ?? 0) > 0 ||
        state.corpFilter != null ||
        state.allianceFilter != null ||
        (state.selectedCorps?.size ?? 0) > 0 ||
        (state.selectedAlliances?.size ?? 0) > 0
    )
}

export function filterPilots(
    pilots: PilotIntel[],
    state: PilotFilterState,
    // Injectable so pure tests don't need the intel store behind
    // getPilotTagStrings.
    getTags: (pilot: PilotIntel) => string[] = getPilotTagStrings
): PilotIntel[] {
    if (!hasActiveFilters(state)) {
        return pilots
    }

    const {
        threatFilter,
        selectedTags,
        corpFilter,
        allianceFilter,
        selectedCorps,
        selectedAlliances,
    } = state

    return pilots.filter((p) => {
        if (threatFilter && p.threat_level.toLowerCase() !== threatFilter) {
            return false
        }

        if (corpFilter && p.character.corporation_ticker !== corpFilter) {
            return false
        }

        if (allianceFilter && p.character.alliance_ticker !== allianceFilter) {
            return false
        }

        if (
            selectedCorps &&
            selectedCorps.size > 0 &&
            !selectedCorps.has(p.character.corporation_name || 'Unknown')
        ) {
            return false
        }

        if (
            selectedAlliances &&
            selectedAlliances.size > 0 &&
            !(
                p.character.alliance_name != null &&
                selectedAlliances.has(p.character.alliance_name)
            )
        ) {
            return false
        }

        if (
            selectedTags &&
            selectedTags.size > 0 &&
            !getTags(p).some((tag) => selectedTags.has(tag))
        ) {
            return false
        }

        return true
    })
}
