<script setup lang="ts">
import { ref, computed } from 'vue'
import type { PilotIntel } from '../types'
import PilotRow from './PilotRow.vue'
import SortableHeader from './SortableHeader.vue'
import { useSettings } from '../composables/useSettings'
import { usePilotSort } from '../composables/usePilotSort'

const props = defineProps<{
    pilots: PilotIntel[]
}>()

const { settings } = useSettings()

const expandedPilot = ref<number | null>(null)

const sortKey = computed({
    get: () => settings.value.sortColumn,
    set: (val) => {
        settings.value.sortColumn = val
    },
})
const sortDirection = computed({
    get: () => settings.value.sortDirection,
    set: (val) => {
        settings.value.sortDirection = val
    },
})

const pilotsRef = computed(() => props.pilots)
const { handleSort, sortedPilots } = usePilotSort(pilotsRef, {
    sortKey,
    sortDirection,
})

function toggleExpand(id: number) {
    expandedPilot.value = expandedPilot.value === id ? null : id
}

function rowKey(pilot: PilotIntel): string | number {
    return pilot.character.id || pilot.character.name
}
</script>

<template>
    <main class="flex-1 flex flex-col overflow-hidden">
        <!-- Header sits outside the scroll container so the scrollbar starts
             below it. overflow-hidden + scrollbar-gutter reserve the same
             gutter as the list, keeping the columns aligned. -->
        <div
            class="pilot-grid flex-none overflow-hidden [scrollbar-gutter:stable] bg-eve-bg-2 border-b border-eve-border text-[10px] font-semibold tracking-wider text-eve-text-3 uppercase"
        >
            <div class="px-2 py-2 text-left">
                <SortableHeader
                    label="Threat"
                    sort-key="threat"
                    :current-sort="sortKey"
                    :sort-direction="sortDirection"
                    @sort="handleSort"
                />
            </div>
            <div class="px-2 py-2 text-left min-w-0">
                <SortableHeader
                    sort-key="pilot"
                    :current-sort="sortKey"
                    :sort-direction="sortDirection"
                    @sort="handleSort"
                >
                    <span class="flex items-center gap-2">
                        Pilot
                        <span
                            class="font-mono text-[10px] bg-eve-bg-3 px-1.5 py-0.5 rounded text-eve-cyan"
                            >{{ pilots.length }}</span
                        >
                    </span>
                </SortableHeader>
            </div>
            <div class="px-2 py-2 text-left">Tags</div>
            <div class="px-2 py-2 text-left min-w-0">
                <SortableHeader
                    label="Corporation"
                    sort-key="corporation"
                    :current-sort="sortKey"
                    :sort-direction="sortDirection"
                    @sort="handleSort"
                />
            </div>
            <div class="px-2 py-2 text-left min-w-0">
                <SortableHeader
                    label="Alliance"
                    sort-key="alliance"
                    :current-sort="sortKey"
                    :sort-direction="sortDirection"
                    @sort="handleSort"
                />
            </div>
            <div class="px-2 py-2 text-left">Ships</div>
            <div class="px-2 py-2 text-right col-span-2">
                <SortableHeader
                    label="K/D"
                    sort-key="kd"
                    :current-sort="sortKey"
                    :sort-direction="sortDirection"
                    @sort="handleSort"
                />
            </div>
            <div class="px-2 py-2 text-right">
                <SortableHeader
                    label="ISK"
                    sort-key="isk"
                    :current-sort="sortKey"
                    :sort-direction="sortDirection"
                    @sort="handleSort"
                />
            </div>
            <div class="px-2 py-2 text-right">
                <SortableHeader
                    label="PPK"
                    sort-key="ppk"
                    :current-sort="sortKey"
                    :sort-direction="sortDirection"
                    align="right"
                    @sort="handleSort"
                />
            </div>
            <div class="px-2 py-2 text-right">
                <SortableHeader
                    label="CPK"
                    sort-key="cpk"
                    :current-sort="sortKey"
                    :sort-direction="sortDirection"
                    align="right"
                    @sort="handleSort"
                />
            </div>
            <div class="px-2 py-2 text-right">
                <SortableHeader
                    label="Active"
                    sort-key="active"
                    :current-sort="sortKey"
                    :sort-direction="sortDirection"
                    align="right"
                    @sort="handleSort"
                />
            </div>
        </div>

        <!-- Rows scroll independently of the header; rows shared between two
             scans keep their key and stay put, others cross-fade. Results
             always stream in paced batches (see commands/lookup.rs), so the
             same fade/slide animations run for cached and fresh scans. -->
        <div class="relative flex-1 overflow-y-auto [scrollbar-gutter:stable]">
            <TransitionGroup name="row-fade">
                <PilotRow
                    v-for="pilot in sortedPilots"
                    :key="rowKey(pilot)"
                    :pilot="pilot"
                    :expanded="expandedPilot === pilot.character.id"
                    @toggle="toggleExpand(pilot.character.id)"
                />
            </TransitionGroup>
        </div>
    </main>
</template>
