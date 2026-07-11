import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { PilotIntel } from '../types'
import { lookupIntel, clearEntries, isAuthenticated } from '../stores/intel'
import { splitPilotNames } from '../utils/scanInput'
import { usePilotStream } from './usePilotStream'
import { useScanFilters } from './useScanFilters'
import { useScanListeners, type LookupProgress } from './useScanListeners'

export type { LookupProgress } from './useScanListeners'

/**
 * Public facade for the main window's scan pipeline. Composes:
 * - usePilotStream — result accumulation, throttled flush, overlay mirror
 * - useScanFilters — filter state, cross-window sync, filtered list
 * - useScanListeners — Tauri event plumbing with disposal guards
 * and keeps only the lookup orchestration here.
 */
export function usePilots() {
    const pilotNames = ref('')
    const loading = ref(false)
    const error = ref<string | null>(null)
    const progress = ref<LookupProgress | null>(null)

    const stream = usePilotStream()
    const filters = useScanFilters(stream.pilots)

    const listeners = useScanListeners({
        onPilotBatch: (batch) => {
            // No intermediate array and no re-throttle: the backend already
            // paces batches, so ingest and flush once per event.
            for (const result of batch.pilots) {
                stream.upsert(result.pilot)
            }
            progress.value = batch.progress
            stream.flush()
        },
        onOverlaySyncRequest: () => stream.broadcastToOverlay(),
        onFilterSync: (state) => filters.applySyncedState(state),
    })

    const pilotCount = computed(() => splitPilotNames(pilotNames.value).length)

    async function lookupPilots(namesOverride?: string) {
        const names = namesOverride ?? pilotNames.value
        if (!names.trim()) return

        if (namesOverride) {
            pilotNames.value = namesOverride
        }

        // Ensure the pilot-batch listener exists before results can stream.
        await listeners.ready

        loading.value = true
        stream.retainScan(names)
        error.value = null
        progress.value = null
        clearEntries()
        filters.clearFilters()

        try {
            const finalPilots = await invoke<PilotIntel[]>('lookup_pilots', {
                namesText: names,
            })

            // Event delivery isn't guaranteed to complete before the invoke
            // resolves, so merge the authoritative return value — upserting
            // by character id makes overlap with streamed batches safe. This
            // ensures every pilot is visible before the intel lookup below.
            stream.upsertAll(finalPilots)
            stream.flush()

            // After scan completes, look up intel for all scanned entities
            if (isAuthenticated.value) {
                const entityIds = stream.pilots.value.flatMap((p) =>
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
            stream.flush()
            loading.value = false
            progress.value = null
        }
    }

    function clear() {
        pilotNames.value = ''
        stream.clear()
        error.value = null
        progress.value = null
        filters.clearFilters()
    }

    function cleanup() {
        listeners.cleanup()
        filters.dispose()
    }

    return {
        pilotNames,
        pilots: stream.pilots,
        filteredPilots: filters.filteredPilots,
        loading,
        error,
        progress,
        pilotCount,
        threatFilter: filters.threatFilter,
        selectedCorps: filters.selectedCorps,
        selectedAlliances: filters.selectedAlliances,
        corpFilter: filters.corpFilter,
        allianceFilter: filters.allianceFilter,
        selectedTags: filters.selectedTags,
        lookupPilots,
        toggleThreatFilter: filters.toggleThreatFilter,
        toggleCorp: filters.toggleCorp,
        toggleAlliance: filters.toggleAlliance,
        toggleCorpFilter: filters.toggleCorpFilter,
        toggleAllianceFilter: filters.toggleAllianceFilter,
        toggleTag: filters.toggleTag,
        clearFilters: filters.clearFilters,
        clear,
        cleanup,
    }
}
