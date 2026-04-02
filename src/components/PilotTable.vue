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
</script>

<template>
    <main class="flex-1 flex flex-col overflow-hidden">
        <div class="flex-1 overflow-y-scroll relative">
            <table class="w-full border-separate border-spacing-0">
                <thead
                    class="sticky top-0 z-10 bg-eve-bg-2 text-[10px] font-semibold tracking-wider text-eve-text-3 uppercase"
                >
                    <tr>
                        <th
                            class="px-2 py-2 text-left border-b border-eve-border bg-eve-bg-2"
                        >
                            <SortableHeader
                                label="Threat"
                                sort-key="threat"
                                :current-sort="sortKey"
                                :sort-direction="sortDirection"
                                @sort="handleSort"
                            />
                        </th>
                        <th
                            class="px-2 py-2 text-left border-b border-eve-border bg-eve-bg-2"
                        >
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
                        </th>
                        <th
                            class="px-2 py-2 text-left border-b border-eve-border bg-eve-bg-2"
                        >
                            Tags
                        </th>
                        <th
                            class="px-2 py-2 text-left border-b border-eve-border bg-eve-bg-2"
                        >
                            <SortableHeader
                                label="Corporation"
                                sort-key="corporation"
                                :current-sort="sortKey"
                                :sort-direction="sortDirection"
                                @sort="handleSort"
                            />
                        </th>
                        <th
                            class="px-2 py-2 text-left border-b border-eve-border bg-eve-bg-2"
                        >
                            <SortableHeader
                                label="Alliance"
                                sort-key="alliance"
                                :current-sort="sortKey"
                                :sort-direction="sortDirection"
                                @sort="handleSort"
                            />
                        </th>
                        <th
                            class="px-2 py-2 text-left border-b border-eve-border bg-eve-bg-2"
                        >
                            Ships
                        </th>
                        <th class="px-2 py-2 text-right" colspan="2">
                            <SortableHeader
                                label="K/D"
                                sort-key="kd"
                                :current-sort="sortKey"
                                :sort-direction="sortDirection"
                                @sort="handleSort"
                            />
                        </th>
                        <th
                            class="px-2 py-2 text-right border-b border-eve-border bg-eve-bg-2"
                        >
                            <SortableHeader
                                label="ISK"
                                sort-key="isk"
                                :current-sort="sortKey"
                                :sort-direction="sortDirection"
                                @sort="handleSort"
                            />
                        </th>
                        <th
                            class="px-2 py-2 text-right border-b border-eve-border bg-eve-bg-2"
                        >
                            <SortableHeader
                                label="PPK"
                                sort-key="ppk"
                                :current-sort="sortKey"
                                :sort-direction="sortDirection"
                                align="right"
                                @sort="handleSort"
                            />
                        </th>
                        <th
                            class="px-2 py-2 text-right border-b border-eve-border bg-eve-bg-2"
                        >
                            <SortableHeader
                                label="CPK"
                                sort-key="cpk"
                                :current-sort="sortKey"
                                :sort-direction="sortDirection"
                                align="right"
                                @sort="handleSort"
                            />
                        </th>
                        <th
                            class="px-2 py-2 text-right border-b border-eve-border bg-eve-bg-2"
                        >
                            <SortableHeader
                                label="Active"
                                sort-key="active"
                                :current-sort="sortKey"
                                :sort-direction="sortDirection"
                                align="right"
                                @sort="handleSort"
                            />
                        </th>
                    </tr>
                </thead>
                <PilotRow
                    v-for="pilot in sortedPilots"
                    :key="pilot.character.id || pilot.character.name"
                    :pilot="pilot"
                    :expanded="expandedPilot === pilot.character.id"
                    @toggle="toggleExpand(pilot.character.id)"
                />
            </table>
        </div>
    </main>
</template>

<style>
.row-move,
.row-enter-active,
.row-leave-active {
    transition: all 0.3s ease;
}

.row-enter-from {
    opacity: 0;
    transform: translateX(-20px);
}

.row-leave-to {
    opacity: 0;
}

.row-leave-active {
    position: absolute;
    left: 0;
    right: 0;
}
</style>
