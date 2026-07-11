import { ref, computed, type Ref, type WritableComputedRef } from 'vue'
import type { PilotIntel } from '../types'
import { sortPilots, type SortDirection } from '../utils/pilotSort'

export interface UsePilotSortOptions {
    sortKey?: Ref<string> | WritableComputedRef<string>
    sortDirection?: Ref<SortDirection> | WritableComputedRef<SortDirection>
}

export function usePilotSort(
    pilots: Ref<PilotIntel[]>,
    options?: UsePilotSortOptions
) {
    const sortKey = options?.sortKey ?? ref<string>('threat')
    const sortDirection = options?.sortDirection ?? ref<SortDirection>('desc')

    function handleSort(key: string) {
        if (sortKey.value === key) {
            sortDirection.value = sortDirection.value === 'asc' ? 'desc' : 'asc'
        } else {
            sortKey.value = key
            sortDirection.value = 'desc'
        }
    }

    const sortedPilots = computed(() =>
        sortPilots(pilots.value, sortKey.value, sortDirection.value)
    )

    return {
        sortKey,
        sortDirection,
        handleSort,
        sortedPilots,
    }
}
