import { ref, shallowRef, computed, reactive, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, emit, type UnlistenFn } from '@tauri-apps/api/event'
import { useThrottleFn, tryOnScopeDispose } from '@vueuse/core'
import type { PilotIntel } from '../types'
import {
    lookupIntel,
    clearEntries,
    isAuthenticated,
    entries,
} from '../stores/intel'
import { getPilotTagStrings } from '../utils/pilotTags'
import { PilotAccumulator } from '../utils/pilotAccumulator'
import { useOverlayWindow } from './useOverlayWindow'

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

// How often (ms) streamed results are flushed to the table and mirrored to
// the overlay window. Keeps large scans from re-sorting and re-broadcasting
// the whole array on every single pilot result.
const SYNC_THROTTLE_MS = 100

export function usePilots() {
    const pilotNames = ref('')
    // shallowRef: the array is only ever reassigned wholesale in
    // flushPilots(), so deep reactivity over every pilot object is wasted
    // work. Row components receive plain objects and never mutate them.
    const pilots = shallowRef<PilotIntel[]>([])
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
    let isSyncingFilters = false
    // Set by cleanup(). setupListeners() checks it after every await so a
    // listener that registers after the scope was disposed is immediately
    // unsubscribed instead of leaking.
    let disposed = false

    const threatFilter = ref<string | null>(null)
    const corpFilter = ref<string | null>(null)
    const allianceFilter = ref<string | null>(null)

    // Streamed pilot results accumulate here (O(1) dedup + insert) and are
    // flushed into the reactive `pilots` array at most every SYNC_THROTTLE_MS.
    const accumulator = new PilotAccumulator()

    // Shared module-level ref: tracks whether the overlay window exists so
    // flushes can skip the cross-window IPC broadcast when nobody listens.
    const { isOverlayOpen } = useOverlayWindow()

    // Flush accumulated results into the reactive array and mirror them to
    // the overlay window in the same step, so both views stay in lockstep.
    // The array is insertion-ordered; each view applies its own sort.
    function flushPilots() {
        pilots.value = accumulator.toArray()
        // Only broadcast when the overlay exists — when it (re)opens it
        // emits 'overlay-sync-request' and gets hydrated from the listener
        // below, so skipped flushes are never lost.
        if (isOverlayOpen.value) {
            emit('pilots-sync', pilots.value)
        }
    }

    const throttledFlush = useThrottleFn(flushPilots, SYNC_THROTTLE_MS, true)

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
        const onProgress = await listen<LookupProgress>(
            'lookup-progress',
            (event) => {
                progress.value = event.payload
            }
        )
        if (disposed) {
            onProgress()
            return
        }
        unlistenProgress = onProgress

        const onPilotResult = await listen<PilotResult>(
            'pilot-result',
            (event) => {
                accumulator.upsert(event.payload.pilot)
                throttledFlush()
            }
        )
        if (disposed) {
            onPilotResult()
            return
        }
        unlistenPilotResult = onPilotResult

        const onSyncRequest = await listen('overlay-sync-request', () => {
            emit('pilots-sync', pilots.value)
        })
        if (disposed) {
            onSyncRequest()
            return
        }
        unlistenSyncRequest = onSyncRequest

        const onFilterSync = await listen<FilterState>(
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
        if (disposed) {
            onFilterSync()
            return
        }
        unlistenFilterSync = onFilterSync
    }

    // Lookups await this so results streamed right after startup (e.g. from
    // a deep link) can't arrive before the pilot-result listener exists.
    const listenersReady = setupListeners()

    const pilotCount = computed(() => {
        return pilotNames.value.split('\n').filter((n) => n.trim()).length
    })

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

        await listenersReady

        loading.value = true
        // Cross-fade between scans: keep rows whose pilot is also in the new
        // scan (fresh data streams in under the same key, updating in place),
        // drop only the rows that left (they fade out), and let new pilots
        // fade in as their results arrive — instead of blanking the list.
        const newNames = new Set(
            names
                .split('\n')
                .map((n) => n.trim().toLowerCase())
                .filter(Boolean)
        )
        accumulator.retainWhere((p) =>
            newNames.has(p.character.name.toLowerCase())
        )
        flushPilots()
        error.value = null
        progress.value = null
        clearEntries()
        clearFilters()

        try {
            const finalPilots = await invoke<PilotIntel[]>('lookup_pilots', {
                namesText: names,
            })

            // Event delivery isn't guaranteed to complete before the invoke
            // resolves, so merge the authoritative return value — upserting
            // by character id makes overlap with streamed events safe. This
            // ensures every pilot is visible before the intel lookup below.
            for (const pilot of finalPilots) {
                accumulator.upsert(pilot)
            }
            flushPilots()

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
            flushPilots()
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
        accumulator.clear()
        flushPilots()
        error.value = null
        progress.value = null
        clearFilters()
    }

    function cleanup() {
        disposed = true
        unlistenProgress?.()
        unlistenPilotResult?.()
        unlistenSyncRequest?.()
        unlistenFilterSync?.()
        unlistenProgress = null
        unlistenPilotResult = null
        unlistenSyncRequest = null
        unlistenFilterSync = null
        stopTagPrune()
    }

    tryOnScopeDispose(cleanup)

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
