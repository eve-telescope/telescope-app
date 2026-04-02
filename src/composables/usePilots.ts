import { ref, computed, reactive, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, emit, type UnlistenFn } from '@tauri-apps/api/event'
import type { PilotIntel } from '../types'
import {
    lookupIntel,
    clearEntries,
    isAuthenticated,
    entries,
} from '../stores/intel'
import { getPilotTagStrings } from '../utils/pilotTags'

export interface LookupProgress {
    current: number
    total: number
    cache_hits: number
}

interface PilotResult {
    pilot: PilotIntel
    index: number
}

interface FilterState {
    threatFilter: string | null
    selectedTags: string[]
    selectedCorps: string[]
    selectedAlliances: string[]
}

const FILTER_SYNC_EVENT = 'filter-state-sync'

const THREAT_ORDER: Record<string, number> = {
    EXTREME: 0,
    HIGH: 1,
    MODERATE: 2,
    LOW: 3,
    MINIMAL: 4,
    Unknown: 5,
}

function insertSorted(pilots: PilotIntel[], pilot: PilotIntel): PilotIntel[] {
    if (pilots.some((p) => p.character.id === pilot.character.id)) {
        return pilots
    }

    const newPilots = [...pilots]
    const pilotOrder = THREAT_ORDER[pilot.threat_level] ?? 5

    let insertIndex = newPilots.findIndex(
        (p) => (THREAT_ORDER[p.threat_level] ?? 5) > pilotOrder
    )

    if (insertIndex === -1) {
        insertIndex = newPilots.length
    }

    newPilots.splice(insertIndex, 0, pilot)
    return newPilots
}

export function usePilots() {
    const pilotNames = ref('')
    const pilots = ref<PilotIntel[]>([])
    const loading = ref(false)
    const error = ref<string | null>(null)
    const progress = ref<LookupProgress | null>(null)

    const selectedCorps = reactive(new Set<string>())
    const selectedAlliances = reactive(new Set<string>())
    const selectedTags = reactive(new Set<string>())

    let unlistenProgress: UnlistenFn | null = null
    let unlistenPilotResult: UnlistenFn | null = null
    let unlistenSyncRequest: UnlistenFn | null = null
    let unlistenFilterSync: UnlistenFn | null = null
    let listenersSetup = false
    let isSyncingFilters = false

    const threatFilter = ref<string | null>(null)
    const corpFilter = ref<string | null>(null)
    const allianceFilter = ref<string | null>(null)

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

    async function setupListeners() {
        if (listenersSetup) return

        unlistenProgress?.()
        unlistenPilotResult?.()
        unlistenSyncRequest?.()
        unlistenFilterSync?.()

        unlistenProgress = await listen<LookupProgress>(
            'lookup-progress',
            (event) => {
                progress.value = event.payload
            }
        )

        unlistenPilotResult = await listen<PilotResult>(
            'pilot-result',
            (event) => {
                pilots.value = insertSorted(pilots.value, event.payload.pilot)
            }
        )

        unlistenSyncRequest = await listen('overlay-sync-request', () => {
            emit('pilots-sync', pilots.value)
        })

        unlistenFilterSync = await listen<FilterState>(
            FILTER_SYNC_EVENT,
            (event) => {
                isSyncingFilters = true
                threatFilter.value = event.payload.threatFilter

                selectedTags.clear()
                for (const tag of event.payload.selectedTags) {
                    selectedTags.add(tag)
                }

                selectedCorps.clear()
                for (const name of event.payload.selectedCorps) {
                    selectedCorps.add(name)
                }

                selectedAlliances.clear()
                for (const name of event.payload.selectedAlliances) {
                    selectedAlliances.add(name)
                }

                isSyncingFilters = false
            }
        )

        listenersSetup = true
    }

    setupListeners()

    watch(
        pilots,
        (newPilots) => {
            emit('pilots-sync', newPilots)
        },
        { deep: true }
    )

    const pilotCount = computed(() => {
        return pilotNames.value.split('\n').filter((n) => n.trim()).length
    })

    // Prune stale tag filters when intel entries change
    watch(entries, () => {
        if (selectedTags.size === 0 || pilots.value.length === 0) return
        const availableTags = new Set(pilots.value.flatMap(getPilotTagStrings))
        for (const tag of selectedTags) {
            if (!availableTags.has(tag)) {
                selectedTags.delete(tag)
            }
        }
    })

    const filteredPilots = computed(() => {
        const hasAnyFilter =
            threatFilter.value !== null ||
            selectedTags.size > 0 ||
            corpFilter.value !== null ||
            allianceFilter.value !== null ||
            selectedCorps.size > 0 ||
            selectedAlliances.size > 0

        if (!hasAnyFilter) {
            return pilots.value
        }

        return pilots.value.filter((p) => {
            if (
                threatFilter.value &&
                p.threat_level.toLowerCase() !== threatFilter.value
            ) {
                return false
            }

            const corpMatch =
                selectedCorps.size === 0 ||
                selectedCorps.has(p.character.corporation_name || 'Unknown')

            const allianceMatch =
                selectedAlliances.size === 0 ||
                (p.character.alliance_name != null &&
                    selectedAlliances.has(p.character.alliance_name))

            const tagMatch =
                selectedTags.size === 0 ||
                getPilotTagStrings(p).some((tag) => selectedTags.has(tag))

            return corpMatch && allianceMatch && tagMatch
        })
    })

    async function lookupPilots(namesOverride?: string) {
        const names = namesOverride ?? pilotNames.value
        if (!names.trim()) return

        if (namesOverride) {
            pilotNames.value = namesOverride
        }

        loading.value = true
        pilots.value = []
        error.value = null
        progress.value = null
        clearEntries()
        clearFilters()

        try {
            await invoke('lookup_pilots', {
                namesText: names,
            })

            // After scan completes, look up intel for all scanned entities
            if (isAuthenticated.value) {
                const entityIds = pilots.value.flatMap((p) =>
                    [
                        p.character.id,
                        p.character.corporation_id,
                        p.character.alliance_id,
                    ].filter((id): id is number => id != null)
                )
                if (entityIds.length > 0) {
                    await lookupIntel([...new Set(entityIds)])
                }
            }
        } catch (e) {
            console.error('Failed to lookup pilots:', e)
            error.value = String(e)
        } finally {
            loading.value = false
            progress.value = null
        }
    }

    function toggleThreatFilter(level: string) {
        threatFilter.value = threatFilter.value === level ? null : level
        broadcastFilterState()
    }

    function toggleCorp(name: string) {
        if (selectedCorps.has(name)) {
            selectedCorps.delete(name)
        } else {
            selectedCorps.add(name)
        }
        broadcastFilterState()
    }

    function toggleAlliance(name: string) {
        if (selectedAlliances.has(name)) {
            selectedAlliances.delete(name)
        } else {
            selectedAlliances.add(name)
        }
        broadcastFilterState()
    }

    function toggleCorpFilter(ticker: string) {
        corpFilter.value = corpFilter.value === ticker ? null : ticker
        broadcastFilterState()
    }

    function toggleAllianceFilter(ticker: string) {
        allianceFilter.value = allianceFilter.value === ticker ? null : ticker
        broadcastFilterState()
    }

    function toggleTag(tag: string) {
        if (selectedTags.has(tag)) {
            selectedTags.delete(tag)
        } else {
            selectedTags.add(tag)
        }
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

    function clear() {
        pilotNames.value = ''
        pilots.value = []
        error.value = null
        progress.value = null
        clearFilters()
    }

    function cleanup() {
        unlistenProgress?.()
        unlistenPilotResult?.()
    }

    return {
        pilotNames,
        pilots,
        filteredPilots,
        loading,
        error,
        progress,
        pilotCount,
        threatFilter,
        selectedCorps,
        selectedAlliances,
        corpFilter,
        allianceFilter,
        selectedTags,
        lookupPilots,
        toggleThreatFilter,
        toggleCorp,
        toggleAlliance,
        toggleCorpFilter,
        toggleAllianceFilter,
        toggleTag,
        clearFilters,
        clear,
        cleanup,
    }
}
