import { ref, computed, onMounted, onUnmounted, type Ref } from 'vue'
import { emit, listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { PilotIntel } from '../types'

export interface FilterState {
    threatFilter: string | null
    selectedTags: string[]
    corpFilter: string | null
    allianceFilter: string | null
}

const FILTER_SYNC_EVENT = 'filter-state-sync'

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
        corpFilter.value = state.corpFilter
        allianceFilter.value = state.allianceFilter
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

    const hasFilters = computed(() => {
        return (
            threatFilter.value !== null ||
            selectedTags.value.size > 0 ||
            corpFilter.value !== null ||
            allianceFilter.value !== null
        )
    })

    const filteredPilots = computed(() => {
        let result = pilots.value

        if (threatFilter.value) {
            result = result.filter(
                (p) => p.threat_level.toLowerCase() === threatFilter.value
            )
        }

        if (selectedTags.value.size > 0) {
            result = result.filter((p) => {
                const flags = p.flags
                if (selectedTags.value.has('super') && flags.is_super)
                    return true
                if (selectedTags.value.has('capital') && flags.is_capital)
                    return true
                if (selectedTags.value.has('blops') && flags.is_blops)
                    return true
                if (selectedTags.value.has('recon') && flags.is_recon)
                    return true
                if (selectedTags.value.has('cyno') && flags.is_cyno) return true
                if (selectedTags.value.has('solo') && flags.is_solo) return true
                return false
            })
        }

        if (corpFilter.value) {
            result = result.filter(
                (p) => p.character.corporation_ticker === corpFilter.value
            )
        }

        if (allianceFilter.value) {
            result = result.filter(
                (p) => p.character.alliance_ticker === allianceFilter.value
            )
        }

        return result
    })

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
