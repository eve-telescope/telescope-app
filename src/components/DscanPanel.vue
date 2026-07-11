<script setup lang="ts">
import { computed, ref } from 'vue'
import { Radar, Ship, Boxes, X } from 'lucide-vue-next'
import { Checkbox } from '@/components/ui/checkbox'
import { getShipIconUrl } from '../utils/format'
import {
    barWidth as barWidthOf,
    bucketByType,
    classIcon,
    countByClass,
} from '../utils/dscan'
import type { DscanParseResult } from '../types'

const props = defineProps<{
    rawInput: string
    result: DscanParseResult | null
    loading: boolean
    error: string | null
}>()

const showOther = ref(false)
const selectedClass = ref<string | null>(null)

function toggleClass(name: string) {
    selectedClass.value = selectedClass.value === name ? null : name
}

const shipEntries = computed(
    () => props.result?.entries.filter((e) => e.is_ship) ?? []
)
const otherEntries = computed(
    () => props.result?.entries.filter((e) => !e.is_ship) ?? []
)

// "24 Sabres", "30 Drakes" — instances per specific ship type.
const shipTypes = computed(() =>
    bucketByType(shipEntries.value, (e) => e.group_name ?? 'Unknown class')
)

// Ship types filtered by the selected class (subtitle is the group name).
const visibleShipTypes = computed(() =>
    selectedClass.value
        ? shipTypes.value.filter((t) => t.subtitle === selectedClass.value)
        : shipTypes.value
)

// "20 Cruisers", "25 Logistics" — instances per ship class.
const shipClasses = computed(() => countByClass(shipEntries.value))

const maxClassCount = computed(() =>
    shipClasses.value.reduce((max, c) => Math.max(max, c.count), 0)
)

function classBarWidth(count: number): string {
    return barWidthOf(count, maxClassCount.value, 4)
}

const otherTypes = computed(() =>
    bucketByType(otherEntries.value, (e) => e.category_name ?? 'Unknown')
)

const maxShipCount = computed(() =>
    shipTypes.value.reduce((max, t) => Math.max(max, t.count), 0)
)

function barWidth(count: number): string {
    return barWidthOf(count, maxShipCount.value, 6)
}
</script>

<template>
    <section class="flex h-full flex-col overflow-hidden bg-eve-bg-0">
        <div
            v-if="error"
            class="border-b border-eve-red/20 bg-eve-red/8 px-5 py-3 text-sm text-eve-red"
        >
            {{ error }}
        </div>

        <!-- Empty. Parsing is near-instant (in-memory SDE index), so there is
             no loading state: previous results stay visible until replaced. -->
        <div
            v-if="!result"
            class="flex h-full flex-col items-center justify-center gap-4 text-eve-text-3"
        >
            <Radar class="h-16 w-16 opacity-20" :stroke-width="1" />
            <div class="text-center">
                <p class="mb-1 text-sm text-eve-text-2">No D-scan results</p>
                <p class="text-xs">Paste directional scan output to begin</p>
            </div>
        </div>

        <!-- Results -->
        <div v-else class="flex min-h-0 flex-1 flex-col overflow-hidden">
            <!-- Headline counts -->
            <div
                class="flex flex-none items-end gap-8 border-b border-eve-border px-5 py-4"
            >
                <div>
                    <div
                        class="flex items-center gap-1.5 text-[10px] font-semibold tracking-[0.15em] text-eve-text-3"
                    >
                        <Ship class="h-3.5 w-3.5" />
                        SHIPS
                    </div>
                    <div class="mt-1 text-3xl font-semibold text-eve-cyan">
                        {{ shipEntries.length }}
                    </div>
                </div>
                <div>
                    <div
                        class="text-[10px] font-semibold tracking-[0.15em] text-eve-text-3"
                    >
                        SHIP TYPES
                    </div>
                    <div class="mt-1 text-3xl font-semibold text-eve-text-1">
                        {{ shipTypes.length }}
                    </div>
                </div>
                <div>
                    <div
                        class="text-[10px] font-semibold tracking-[0.15em] text-eve-text-3"
                    >
                        CLASSES
                    </div>
                    <div class="mt-1 text-3xl font-semibold text-eve-text-1">
                        {{ shipClasses.length }}
                    </div>
                </div>
            </div>

            <div
                class="grid min-h-0 flex-1 overflow-hidden lg:grid-cols-[1.6fr_1fr]"
            >
                <!-- Ship types: the primary view -->
                <div class="flex min-h-0 flex-col overflow-hidden">
                    <div
                        class="flex flex-none items-center gap-2 px-5 pt-5 pb-3"
                    >
                        <Radar class="h-4 w-4 text-eve-cyan" />
                        <h3
                            class="text-[10px] font-semibold tracking-[0.15em] text-eve-text-3"
                        >
                            SHIP TYPES
                        </h3>
                        <button
                            v-if="selectedClass"
                            class="ml-1 flex items-center gap-1 rounded-full border border-eve-cyan/40 bg-eve-cyan/10 py-0.5 pr-1.5 pl-2 text-[11px] font-medium text-eve-cyan transition-colors hover:bg-eve-cyan/20"
                            @click="selectedClass = null"
                        >
                            {{ selectedClass }}
                            <X class="h-3 w-3" />
                        </button>
                    </div>

                    <div class="min-h-0 flex-1 overflow-auto px-5 pb-4">
                        <p
                            v-if="shipTypes.length === 0"
                            class="text-sm text-eve-text-3"
                        >
                            No ships detected in this scan.
                        </p>

                        <TransitionGroup
                            v-else
                            tag="ul"
                            name="row-fade"
                            class="relative space-y-1"
                        >
                            <li
                                v-for="item in visibleShipTypes"
                                :key="item.type_name"
                                class="relative flex items-center gap-3 overflow-hidden rounded-md bg-eve-bg-1 px-3 py-2"
                            >
                                <div
                                    class="absolute inset-y-0 left-0 bg-eve-cyan/8"
                                    :style="{ width: barWidth(item.count) }"
                                />
                                <img
                                    v-if="item.type_id"
                                    :src="getShipIconUrl(item.type_id, 64)"
                                    :alt="item.type_name"
                                    class="relative z-10 h-8 w-8 flex-none rounded-sm"
                                    loading="lazy"
                                />
                                <div
                                    v-else
                                    class="relative z-10 h-8 w-8 flex-none rounded-sm bg-eve-bg-3"
                                />
                                <div class="relative z-10 min-w-0 flex-1">
                                    <div
                                        class="truncate text-sm text-eve-text-1"
                                    >
                                        {{ item.type_name }}
                                    </div>
                                    <div
                                        class="truncate text-xs text-eve-text-3"
                                    >
                                        {{ item.subtitle }}
                                    </div>
                                </div>
                                <div
                                    class="relative z-10 flex-none text-xl font-semibold tabular-nums text-eve-cyan"
                                >
                                    {{ item.count }}
                                </div>
                            </li>
                        </TransitionGroup>

                        <!-- Other objects (revealed via toggle) -->
                        <template v-if="showOther && otherTypes.length > 0">
                            <div
                                class="mt-5 mb-2 flex items-center gap-2 text-[10px] font-semibold tracking-[0.15em] text-eve-text-3"
                            >
                                <Boxes class="h-3.5 w-3.5" />
                                OTHER OBJECTS
                            </div>
                            <TransitionGroup
                                tag="ul"
                                name="row-fade"
                                class="relative space-y-1"
                            >
                                <li
                                    v-for="item in otherTypes"
                                    :key="item.type_name"
                                    class="flex items-center gap-3 rounded-md px-3 py-1.5 text-eve-text-2"
                                >
                                    <img
                                        v-if="item.type_id"
                                        :src="getShipIconUrl(item.type_id, 64)"
                                        :alt="item.type_name"
                                        class="h-6 w-6 flex-none rounded-sm opacity-70"
                                        loading="lazy"
                                    />
                                    <div
                                        v-else
                                        class="h-6 w-6 flex-none rounded-sm bg-eve-bg-3"
                                    />
                                    <div class="min-w-0 flex-1">
                                        <div class="truncate text-sm">
                                            {{ item.type_name }}
                                        </div>
                                        <div
                                            class="truncate text-xs text-eve-text-3"
                                        >
                                            {{ item.subtitle }}
                                        </div>
                                    </div>
                                    <div
                                        class="flex-none text-sm font-semibold tabular-nums text-eve-text-2"
                                    >
                                        {{ item.count }}
                                    </div>
                                </li>
                            </TransitionGroup>
                        </template>
                    </div>
                </div>

                <!-- By class summary -->
                <div
                    class="flex min-h-0 flex-col overflow-hidden border-t border-eve-border lg:border-t-0 lg:border-l"
                >
                    <h3
                        class="flex-none px-5 pt-5 pb-3 text-[10px] font-semibold tracking-[0.15em] text-eve-text-3"
                    >
                        BY CLASS
                    </h3>
                    <div class="min-h-0 flex-1 overflow-auto px-5 pb-4">
                        <TransitionGroup
                            tag="ul"
                            name="row-fade"
                            class="relative space-y-1"
                        >
                            <li v-for="item in shipClasses" :key="item.name">
                                <button
                                    class="group relative flex w-full items-center gap-2.5 overflow-hidden rounded-md border px-2.5 py-2 text-left transition-colors"
                                    :class="
                                        selectedClass === item.name
                                            ? 'border-eve-cyan/50 bg-eve-cyan/10'
                                            : 'border-transparent hover:border-eve-border hover:bg-eve-bg-1'
                                    "
                                    @click="toggleClass(item.name)"
                                >
                                    <div
                                        class="absolute inset-y-0 left-0"
                                        :class="
                                            selectedClass === item.name
                                                ? 'bg-eve-cyan/10'
                                                : 'bg-eve-bg-2'
                                        "
                                        :style="{
                                            width: classBarWidth(item.count),
                                        }"
                                    />
                                    <component
                                        :is="classIcon(item.name)"
                                        class="relative z-10 h-4 w-4 flex-none"
                                        :class="
                                            selectedClass === item.name
                                                ? 'text-eve-cyan'
                                                : 'text-eve-text-3 group-hover:text-eve-text-2'
                                        "
                                    />
                                    <span
                                        class="relative z-10 min-w-0 flex-1 truncate text-sm"
                                        :class="
                                            selectedClass === item.name
                                                ? 'text-eve-cyan'
                                                : 'text-eve-text-1'
                                        "
                                    >
                                        {{ item.name }}
                                    </span>
                                    <span
                                        class="relative z-10 flex-none text-sm font-semibold tabular-nums"
                                        :class="
                                            selectedClass === item.name
                                                ? 'text-eve-cyan'
                                                : 'text-eve-text-2'
                                        "
                                    >
                                        {{ item.count }}
                                    </span>
                                </button>
                            </li>
                        </TransitionGroup>
                        <p
                            v-if="shipClasses.length === 0"
                            class="text-sm text-eve-text-3"
                        >
                            —
                        </p>
                    </div>
                </div>
            </div>

            <!-- Toggle for non-ship objects -->
            <label
                class="flex flex-none cursor-pointer items-center gap-2 border-t border-eve-border px-5 py-3 text-xs text-eve-text-2 select-none"
            >
                <Checkbox v-model="showOther" />
                Show other objects ({{ otherEntries.length }})
                <span class="text-eve-text-3">— structures, drones, etc.</span>
            </label>
        </div>
    </section>
</template>
