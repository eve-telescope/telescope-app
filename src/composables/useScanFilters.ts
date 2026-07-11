import { ref, reactive, computed, watch, type Ref } from 'vue'
import { emit } from '@tauri-apps/api/event'
import type { PilotIntel } from '../types'
import { entries } from '../stores/intel'
import { getPilotTagStrings } from '../utils/pilotTags'
import { filterPilots } from '../utils/pilotFilters'
import { FILTER_SYNC_EVENT, type FilterSyncState } from './useScanListeners'

/**
 * Main-window filter state: threat/tag/corp/alliance selections, the
 * cross-window sync broadcast, and the filtered pilot list (delegated to
 * the shared utils/pilotFilters core, which the overlay uses too).
 */
export function useScanFilters(pilots: Ref<PilotIntel[]>) {
    const threatFilter = ref<string | null>(null)
    const corpFilter = ref<string | null>(null)
    const allianceFilter = ref<string | null>(null)
    const selectedCorps = reactive(new Set<string>())
    const selectedAlliances = reactive(new Set<string>())
    const selectedTags = reactive(new Set<string>())

    // Guards against re-broadcasting state we just received from the
    // overlay (which would bounce the event back and forth).
    let isSyncingFilters = false

    function broadcastFilterState() {
        if (!isSyncingFilters) {
            emit(FILTER_SYNC_EVENT, {
                threatFilter: threatFilter.value,
                selectedTags: [...selectedTags],
                selectedCorps: [...selectedCorps],
                selectedAlliances: [...selectedAlliances],
            })
        }
    }

    function replaceSet(target: Set<string>, values: string[]) {
        target.clear()
        for (const value of values) {
            target.add(value)
        }
    }

    function applySyncedState(state: FilterSyncState) {
        isSyncingFilters = true
        threatFilter.value = state.threatFilter
        // The overlay's payload carries corpFilter/allianceFilter instead
        // of the multi-select arrays — treat missing arrays as empty.
        replaceSet(selectedTags, state.selectedTags ?? [])
        replaceSet(selectedCorps, state.selectedCorps ?? [])
        replaceSet(selectedAlliances, state.selectedAlliances ?? [])
        isSyncingFilters = false
    }

    // Prune stale tag filters when intel entries change
    const stopTagPrune = watch(entries, () => {
        if (selectedTags.size === 0 || pilots.value.length === 0) return
        const availableTags = new Set(pilots.value.flatMap(getPilotTagStrings))
        for (const tag of selectedTags) {
            if (!availableTags.has(tag)) {
                selectedTags.delete(tag)
            }
        }
    })

    const filteredPilots = computed(() =>
        filterPilots(pilots.value, {
            threatFilter: threatFilter.value,
            selectedTags,
            corpFilter: corpFilter.value,
            allianceFilter: allianceFilter.value,
            selectedCorps,
            selectedAlliances,
        })
    )

    function toggleThreatFilter(level: string) {
        threatFilter.value = threatFilter.value === level ? null : level
        broadcastFilterState()
    }

    function toggleInSet(target: Set<string>, value: string) {
        if (target.has(value)) {
            target.delete(value)
        } else {
            target.add(value)
        }
        broadcastFilterState()
    }

    function toggleCorp(name: string) {
        toggleInSet(selectedCorps, name)
    }

    function toggleAlliance(name: string) {
        toggleInSet(selectedAlliances, name)
    }

    function toggleTag(tag: string) {
        toggleInSet(selectedTags, tag)
    }

    function toggleCorpFilter(ticker: string) {
        corpFilter.value = corpFilter.value === ticker ? null : ticker
        broadcastFilterState()
    }

    function toggleAllianceFilter(ticker: string) {
        allianceFilter.value = allianceFilter.value === ticker ? null : ticker
        broadcastFilterState()
    }

    function clearFilters() {
        threatFilter.value = null
        selectedCorps.clear()
        selectedAlliances.clear()
        corpFilter.value = null
        allianceFilter.value = null
        selectedTags.clear()
        broadcastFilterState()
    }

    function dispose() {
        stopTagPrune()
    }

    return {
        threatFilter,
        corpFilter,
        allianceFilter,
        selectedCorps,
        selectedAlliances,
        selectedTags,
        filteredPilots,
        applySyncedState,
        toggleThreatFilter,
        toggleCorp,
        toggleAlliance,
        toggleTag,
        toggleCorpFilter,
        toggleAllianceFilter,
        clearFilters,
        dispose,
    }
}
