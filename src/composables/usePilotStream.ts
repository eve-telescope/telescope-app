import { shallowRef } from 'vue'
import { emit } from '@tauri-apps/api/event'
import type { PilotIntel } from '../types'
import { PilotAccumulator } from '../utils/pilotAccumulator'
import { splitPilotNames } from '../utils/scanInput'
import { useOverlayWindow } from './useOverlayWindow'

/**
 * Owns the streamed-results pipeline for the main window: results
 * accumulate in a PilotAccumulator (O(1) dedup by character id) and are
 * flushed into the reactive `pilots` array — and mirrored to the overlay —
 * once per incoming batch. Pacing lives backend-side (one 'pilot-batch'
 * per BATCH_INTERVAL_MS); throttling again here would only add latency.
 */
export function usePilotStream() {
    // shallowRef: the array is only ever reassigned wholesale in flush(),
    // so deep reactivity over every pilot object is wasted work. Row
    // components receive plain objects and never mutate them.
    const pilots = shallowRef<PilotIntel[]>([])
    const accumulator = new PilotAccumulator()

    // Shared module-level ref: tracks whether the overlay window exists so
    // flushes can skip the cross-window IPC broadcast when nobody listens.
    const { isOverlayOpen } = useOverlayWindow()

    // Flush accumulated results into the reactive array and mirror them to
    // the overlay window in the same step, so both views stay in lockstep.
    // The array is insertion-ordered; each view applies its own sort.
    function flush() {
        pilots.value = accumulator.toArray()
        // Only broadcast when the overlay exists — when it (re)opens it
        // emits 'overlay-sync-request' and gets hydrated via
        // broadcastToOverlay(), so skipped flushes are never lost.
        if (isOverlayOpen.value) {
            emit('pilots-sync', pilots.value)
        }
    }

    function upsert(pilot: PilotIntel) {
        accumulator.upsert(pilot)
    }

    function upsertAll(newPilots: PilotIntel[]) {
        for (const pilot of newPilots) {
            accumulator.upsert(pilot)
        }
    }

    // Cross-fade between scans: keep rows whose pilot is also in the new
    // scan (fresh data streams in under the same key, updating in place),
    // drop only the rows that left (they fade out), and let new pilots
    // fade in as their results arrive — instead of blanking the list.
    function retainScan(namesText: string) {
        const newNames = new Set(
            splitPilotNames(namesText).map((name) => name.toLowerCase())
        )
        accumulator.retainWhere((p) =>
            newNames.has(p.character.name.toLowerCase())
        )
        flush()
    }

    /** Unconditional mirror, used to answer 'overlay-sync-request'. */
    function broadcastToOverlay() {
        emit('pilots-sync', pilots.value)
    }

    function clear() {
        accumulator.clear()
        flush()
    }

    return {
        pilots,
        flush,
        upsert,
        upsertAll,
        retainScan,
        broadcastToOverlay,
        clear,
    }
}
