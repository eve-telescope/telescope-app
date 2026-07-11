import type { PilotIntel } from '../types'

/**
 * Accumulates pilot results streamed from the backend during a scan.
 * Upserts are O(1) and keyed by character id. The accumulator does not
 * sort — each view (table, overlay) applies its own sort via
 * usePilotSort, so flushes only pay for the array copy.
 */
export class PilotAccumulator {
    private byCharacterId = new Map<number, PilotIntel>()

    /**
     * Inserts the pilot, or replaces the existing entry for the same
     * character id (a re-scan streams fresh data for retained pilots).
     */
    upsert(pilot: PilotIntel): void {
        this.byCharacterId.set(pilot.character.id, pilot)
    }

    /**
     * Drops entries not matching the predicate. Used when a new scan
     * starts: pilots also present in the new scan keep their row (and
     * key) so the table cross-fades instead of rebuilding.
     */
    retainWhere(predicate: (pilot: PilotIntel) => boolean): void {
        for (const [id, pilot] of this.byCharacterId) {
            if (!predicate(pilot)) {
                this.byCharacterId.delete(id)
            }
        }
    }

    /** Snapshot of the accumulated pilots in insertion (arrival) order. */
    toArray(): PilotIntel[] {
        return [...this.byCharacterId.values()]
    }

    clear() {
        this.byCharacterId.clear()
    }

    get size(): number {
        return this.byCharacterId.size
    }
}
