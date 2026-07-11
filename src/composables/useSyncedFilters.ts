import { ref, computed, onMounted, onUnmounted, type Ref } from 'vue'
import { emit, listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { PilotIntel } from '../types'
import { filterPilots, hasActiveFilters } from '../utils/pilotFilters'
import { FILTER_SYNC_EVENT } from './useScanListeners'

/**
 * Overlay-side filter state, kept in sync with the main window over the
 * FILTER_SYNC_EVENT protocol. The payload shape below is frozen — the main
 * window broadcasts a superset (selectedCorps/selectedAlliances) that this
 * side simply ignores. Matching is delegated to the shared
 * utils/pilotFilters core so both windows filter identically.
 */
export interface FilterState {
    threatFilter: string | null
    selectedTags: string[]
    corpFilter: string | null
    allianceFilter: string | null
}

export function useSyncedFilters(pilots: Ref<PilotIntel[]>) {
    const threatFilter = ref<string | null>(null)
    const selectedTags = ref<Set<string>>(new Set())
    const corpFilter = ref<string | null>(null)
    const allianceFilter = ref<string | null>(null)

    let unlisten: UnlistenFn | null = null
    let isSyncing = false

    function getFilterState(): FilterState {
        return {
            threatFilter: threatFilter.value,
            selectedTags: [...selectedTags.value],
            corpFilter: corpFilter.value,
            allianceFilter: allianceFilter.value,
        }
    }

    function applyFilterState(state: FilterState) {
        isSyncing = true
        threatFilter.value = state.threatFilter
        selectedTags.value = new Set(state.selectedTags)
        corpFilter.value = state.corpFilter ?? null
        allianceFilter.value = state.allianceFilter ?? null
        isSyncing = false
    }

    function broadcastState() {
        if (!isSyncing) {
            emit(FILTER_SYNC_EVENT, getFilterState())
        }
    }

    function toggleThreatFilter(level: string) {
        threatFilter.value = threatFilter.value === level ? null : level
        broadcastState()
    }

    function toggleTag(tag: string) {
        const newSet = new Set(selectedTags.value)
        if (newSet.has(tag)) {
            newSet.delete(tag)
        } else {
            newSet.add(tag)
        }
        selectedTags.value = newSet
        broadcastState()
    }

    function toggleCorpFilter(ticker: string) {
        corpFilter.value = corpFilter.value === ticker ? null : ticker
        broadcastState()
    }

    function toggleAllianceFilter(ticker: string) {
        allianceFilter.value = allianceFilter.value === ticker ? null : ticker
        broadcastState()
    }

    function clearFilters() {
        threatFilter.value = null
        selectedTags.value = new Set()
        corpFilter.value = null
        allianceFilter.value = null
        broadcastState()
    }

    const currentFilterState = computed(() => ({
        threatFilter: threatFilter.value,
        selectedTags: selectedTags.value,
        corpFilter: corpFilter.value,
        allianceFilter: allianceFilter.value,
    }))

    const hasFilters = computed(() =>
        hasActiveFilters(currentFilterState.value)
    )

    const filteredPilots = computed(() =>
        filterPilots(pilots.value, currentFilterState.value)
    )

    onMounted(async () => {
        unlisten = await listen<FilterState>(FILTER_SYNC_EVENT, (event) => {
            applyFilterState(event.payload)
        })
    })

    onUnmounted(() => {
        unlisten?.()
    })

    return {
        threatFilter,
        selectedTags,
        corpFilter,
        allianceFilter,
        hasFilters,
        filteredPilots,
        toggleThreatFilter,
        toggleTag,
        toggleCorpFilter,
        toggleAllianceFilter,
        clearFilters,
    }
}
