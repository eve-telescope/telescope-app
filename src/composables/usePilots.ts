import { ref, computed, reactive, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, emit, type UnlistenFn } from '@tauri-apps/api/event'
import type { PilotIntel } from '../types'

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
    corpFilter: string | null
    allianceFilter: string | null
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
                corpFilter: corpFilter.value,
                allianceFilter: allianceFilter.value,
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

                corpFilter.value = event.payload.corpFilter
                selectedCorps.clear()
                if (event.payload.corpFilter) {
                    const pilot = pilots.value.find(
                        (p) =>
                            p.character.corporation_ticker ===
                            event.payload.corpFilter
                    )
                    if (pilot?.character.corporation_name) {
                        selectedCorps.add(pilot.character.corporation_name)
                    }
                }

                allianceFilter.value = event.payload.allianceFilter
                selectedAlliances.clear()
                if (event.payload.allianceFilter) {
                    const pilot = pilots.value.find(
                        (p) =>
                            p.character.alliance_ticker ===
                            event.payload.allianceFilter
                    )
                    if (pilot?.character.alliance_name) {
                        selectedAlliances.add(pilot.character.alliance_name)
                    }
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
                (selectedCorps.size === 0 && !corpFilter.value) ||
                selectedCorps.has(p.character.corporation_name || 'Unknown') ||
                (corpFilter.value &&
                    p.character.corporation_ticker === corpFilter.value)

            const allianceMatch =
                (selectedAlliances.size === 0 && !allianceFilter.value) ||
                (p.character.alliance_name &&
                    selectedAlliances.has(p.character.alliance_name)) ||
                (allianceFilter.value &&
                    p.character.alliance_ticker === allianceFilter.value)

            let tagMatch = selectedTags.size === 0
            if (!tagMatch) {
                const flags = p.flags
                if (selectedTags.has('cyno') && flags.is_cyno) tagMatch = true
                if (selectedTags.has('recon') && flags.is_recon) tagMatch = true
                if (selectedTags.has('blops') && flags.is_blops) tagMatch = true
                if (selectedTags.has('capital') && flags.is_capital)
                    tagMatch = true
                if (selectedTags.has('super') && flags.is_super) tagMatch = true
                if (selectedTags.has('solo') && flags.is_solo) tagMatch = true
            }

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
        clearFilters()

        try {
            await invoke('lookup_pilots', {
                namesText: names,
            })
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
            if (selectedCorps.size === 0) {
                corpFilter.value = null
            }
        } else {
            selectedCorps.clear()
            selectedCorps.add(name)
            const pilot = pilots.value.find(
                (p) => p.character.corporation_name === name
            )
            corpFilter.value = pilot?.character.corporation_ticker || null
        }
        broadcastFilterState()
    }

    function toggleAlliance(name: string) {
        if (selectedAlliances.has(name)) {
            selectedAlliances.delete(name)
            if (selectedAlliances.size === 0) {
                allianceFilter.value = null
            }
        } else {
            selectedAlliances.clear()
            selectedAlliances.add(name)
            const pilot = pilots.value.find(
                (p) => p.character.alliance_name === name
            )
            allianceFilter.value = pilot?.character.alliance_ticker || null
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
