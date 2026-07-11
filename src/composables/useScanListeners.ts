import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { tryOnScopeDispose } from '@vueuse/core'
import type { PilotIntel } from '../types'

// ---------------------------------------------------------------------------
// Backend/cross-window event contract
// ---------------------------------------------------------------------------

/** Progress snapshot embedded in every 'pilot-batch' event. */
export interface LookupProgress {
    current: number
    total: number
    cache_hits: number
}

export interface PilotResult {
    pilot: PilotIntel
    index: number
}

/**
 * Payload of the backend 'pilot-batch' event: all pilots completed since
 * the previous batch plus the progress as of the last one. The backend
 * emits at most one batch per ~100ms (cache hits arrive as one immediate
 * batch, and a final batch always closes the stream).
 */
export interface PilotBatch {
    pilots: PilotResult[]
    progress: LookupProgress
}

/**
 * Cross-window filter sync payload. The main window broadcasts
 * selectedCorps/selectedAlliances; the overlay broadcasts
 * corpFilter/allianceFilter — hence every non-shared field is optional.
 */
export interface FilterSyncState {
    threatFilter: string | null
    selectedTags: string[]
    corpFilter?: string | null
    allianceFilter?: string | null
    selectedCorps?: string[]
    selectedAlliances?: string[]
}

export const FILTER_SYNC_EVENT = 'filter-state-sync'

// ---------------------------------------------------------------------------
// Listener plumbing
// ---------------------------------------------------------------------------

export interface ScanListenerHandlers {
    onPilotBatch: (batch: PilotBatch) => void
    onOverlaySyncRequest: () => void
    onFilterSync: (state: FilterSyncState) => void
}

/**
 * Registers the main window's Tauri event listeners for the scan pipeline.
 * Handles the register-after-dispose race: cleanup() sets a flag that is
 * checked after every await, so a listener that resolves after the scope
 * was disposed is immediately unsubscribed instead of leaking.
 */
export function useScanListeners(handlers: ScanListenerHandlers) {
    let disposed = false
    const unlistenFns: UnlistenFn[] = []

    async function subscribe<T>(
        event: string,
        handler: (payload: T) => void
    ): Promise<boolean> {
        const unlisten = await listen<T>(event, (e) => handler(e.payload))
        if (disposed) {
            unlisten()
            return false
        }
        unlistenFns.push(unlisten)
        return true
    }

    async function setup() {
        if (
            !(await subscribe<PilotBatch>('pilot-batch', handlers.onPilotBatch))
        ) {
            return
        }
        if (
            !(await subscribe<void>(
                'overlay-sync-request',
                handlers.onOverlaySyncRequest
            ))
        ) {
            return
        }
        await subscribe<FilterSyncState>(
            FILTER_SYNC_EVENT,
            handlers.onFilterSync
        )
    }

    // Lookups await this so results streamed right after startup (e.g. from
    // a deep link) can't arrive before the pilot-batch listener exists.
    const ready = setup()

    function cleanup() {
        disposed = true
        for (const unlisten of unlistenFns.splice(0)) {
            unlisten()
        }
    }

    tryOnScopeDispose(cleanup)

    return { ready, cleanup }
}
