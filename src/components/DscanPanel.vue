<script setup lang="ts">
import { computed, ref } from 'vue'
import { Radar, Ship } from 'lucide-vue-next'
import type { DscanParseResult, SdeStatus } from '../types'

type SortKey =
    | 'type_id'
    | 'name'
    | 'type_name'
    | 'group_name'
    | 'category_name'
    | 'distance'

const props = defineProps<{
    rawInput: string
    result: DscanParseResult | null
    sdeStatus?: SdeStatus | null
    loading: boolean
    syncing?: boolean
    error: string | null
}>()

const nonShipCount = computed(
    () => (props.result?.total_rows ?? 0) - (props.result?.ship_count ?? 0)
)

const shipGroupSummary = computed(() => {
    const counts = new Map<string, number>()

    for (const entry of props.result?.entries ?? []) {
        if (!entry.is_ship) {
            continue
        }

        const key = entry.group_name ?? 'Unknown Ship Class'
        counts.set(key, (counts.get(key) ?? 0) + 1)
    }

    return [...counts.entries()]
        .map(([group, count]) => ({ group, count }))
        .sort((a, b) => b.count - a.count || a.group.localeCompare(b.group))
})

const categorySummary = computed(() => {
    const counts = new Map<string, number>()

    for (const entry of props.result?.entries ?? []) {
        const key = entry.category_name ?? 'Unknown'
        counts.set(key, (counts.get(key) ?? 0) + 1)
    }

    return [...counts.entries()]
        .map(([category, count]) => ({ category, count }))
        .sort(
            (a, b) => b.count - a.count || a.category.localeCompare(b.category)
        )
})

const topShipGroups = computed(() => shipGroupSummary.value.slice(0, 8))
const topCategories = computed(() => categorySummary.value.slice(0, 6))

const sortKey = ref<SortKey>('distance')
const sortDirection = ref<'asc' | 'desc'>('asc')

function parseDistanceToMeters(distance: string | null): number {
    if (!distance || distance === '—') {
        return Number.POSITIVE_INFINITY
    }

    const normalized = distance.replace(',', '.').trim()
    const match = normalized.match(/^([\d.]+)\s*(m|km|AU)$/i)
    if (!match) {
        return Number.POSITIVE_INFINITY
    }

    const value = Number.parseFloat(match[1])
    const unit = match[2].toLowerCase()
    if (Number.isNaN(value)) {
        return Number.POSITIVE_INFINITY
    }

    if (unit === 'm') return value
    if (unit === 'km') return value * 1000
    return value * 149_597_870_700
}

function compareValues(a: string | number, b: string | number) {
    if (typeof a === 'number' && typeof b === 'number') {
        return a - b
    }

    return String(a).localeCompare(String(b), undefined, {
        numeric: true,
        sensitivity: 'base',
    })
}

const sortedEntries = computed(() => {
    const entries = [...(props.result?.entries ?? [])]

    entries.sort((a, b) => {
        let comparison = 0

        switch (sortKey.value) {
            case 'type_id':
                comparison = compareValues(
                    a.type_id ?? Number.POSITIVE_INFINITY,
                    b.type_id ?? Number.POSITIVE_INFINITY
                )
                break
            case 'distance':
                comparison = compareValues(
                    parseDistanceToMeters(a.distance),
                    parseDistanceToMeters(b.distance)
                )
                break
            case 'name':
                comparison = compareValues(a.name, b.name)
                break
            case 'type_name':
                comparison = compareValues(a.type_name, b.type_name)
                break
            case 'group_name':
                comparison = compareValues(
                    a.group_name ?? 'Unknown',
                    b.group_name ?? 'Unknown'
                )
                break
            case 'category_name':
                comparison = compareValues(
                    a.category_name ?? 'Unknown',
                    b.category_name ?? 'Unknown'
                )
                break
        }

        if (comparison === 0) {
            comparison = compareValues(a.name, b.name)
        }

        return sortDirection.value === 'asc' ? comparison : -comparison
    })

    return entries
})

function toggleSort(nextKey: SortKey) {
    if (sortKey.value === nextKey) {
        sortDirection.value = sortDirection.value === 'asc' ? 'desc' : 'asc'
        return
    }

    sortKey.value = nextKey
    sortDirection.value = nextKey === 'distance' ? 'asc' : 'desc'
}

function sortIndicator(key: SortKey) {
    if (sortKey.value !== key) {
        return ''
    }

    return sortDirection.value === 'asc' ? ' ↑' : ' ↓'
}
</script>

<template>
    <section class="flex h-full flex-col overflow-hidden bg-eve-bg-0">
        <div class="flex h-full w-full flex-col overflow-hidden">
            <div
                v-if="error"
                class="border-b border-eve-red/20 bg-eve-red/8 px-5 py-3 text-sm text-eve-red"
            >
                {{ error }}
            </div>

            <div
                v-if="result"
                class="grid grid-cols-3 gap-8 border-b border-eve-border px-5 py-4"
            >
                <div>
                    <div
                        class="text-[10px] font-semibold tracking-[0.15em] text-eve-text-3"
                    >
                        TOTAL ROWS
                    </div>
                    <div class="mt-1 text-2xl font-semibold text-eve-text-1">
                        {{ result.total_rows }}
                    </div>
                </div>
                <div>
                    <div
                        class="flex items-center gap-2 text-[10px] font-semibold tracking-[0.15em] text-eve-text-3"
                    >
                        <Ship class="h-3.5 w-3.5" />
                        SHIPS
                    </div>
                    <div class="mt-1 text-2xl font-semibold text-eve-cyan">
                        {{ result.ship_count }}
                    </div>
                </div>
                <div>
                    <div
                        class="text-[10px] font-semibold tracking-[0.15em] text-eve-text-3"
                    >
                        OTHER OBJECTS
                    </div>
                    <div class="mt-1 text-2xl font-semibold text-eve-text-2">
                        {{ nonShipCount }}
                    </div>
                </div>
            </div>

            <div
                v-if="result"
                class="grid flex-none gap-10 px-5 py-5 lg:grid-cols-[1.5fr_1fr]"
            >
                <div>
                    <div class="flex items-center gap-2">
                        <Radar class="h-4 w-4 text-eve-cyan" />
                        <h3
                            class="text-[10px] font-semibold tracking-[0.15em] text-eve-text-3"
                        >
                            SHIP SUMMARY
                        </h3>
                    </div>
                    <div
                        v-if="topShipGroups.length > 0"
                        class="mt-4 grid gap-x-8 gap-y-2 sm:grid-cols-2 xl:grid-cols-3"
                    >
                        <div
                            v-for="item in topShipGroups"
                            :key="item.group"
                            class="flex items-baseline justify-between gap-4"
                        >
                            <span class="text-sm text-eve-text-1">{{
                                item.group
                            }}</span>
                            <span class="text-sm font-semibold text-eve-cyan">{{
                                item.count
                            }}</span>
                        </div>
                    </div>

                    <p v-else class="mt-4 text-sm text-eve-text-3">
                        No ships detected in this scan.
                    </p>
                </div>

                <div>
                    <h3
                        class="text-[10px] font-semibold tracking-[0.15em] text-eve-text-3"
                    >
                        OBJECT MIX
                    </h3>
                    <div class="mt-4 space-y-2">
                        <div
                            v-for="item in topCategories"
                            :key="item.category"
                            class="flex items-baseline justify-between gap-4"
                        >
                            <span class="text-sm text-eve-text-2">{{
                                item.category
                            }}</span>
                            <span
                                class="text-sm font-semibold text-eve-text-1"
                                >{{ item.count }}</span
                            >
                        </div>
                    </div>
                </div>
            </div>

            <div
                class="min-h-0 flex-1 overflow-hidden border-t border-eve-border"
            >
                <div
                    v-if="loading"
                    class="flex h-full items-center justify-center text-sm text-eve-text-3"
                >
                    Parsing directional scan...
                </div>

                <div
                    v-else-if="!result"
                    class="flex h-full flex-col items-center justify-center gap-4 text-eve-text-3"
                >
                    <Radar class="w-16 h-16 opacity-20" :stroke-width="1" />
                    <div class="text-center">
                        <p class="text-sm text-eve-text-2 mb-1">
                            No D-scan results
                        </p>
                        <p class="text-xs">
                            Paste directional scan output to begin
                        </p>
                    </div>
                </div>

                <div v-else class="flex h-full flex-col overflow-hidden">
                    <div class="px-5 py-3 text-xs text-eve-text-3">
                        Raw parsed rows for verification. The summary above is
                        the primary D-scan view.
                    </div>

                    <div class="min-h-0 overflow-hidden px-5 pb-4">
                        <div
                            class="grid h-full min-h-0 overflow-auto text-xs [grid-template-columns:100px_minmax(180px,1.4fr)_minmax(140px,1fr)_minmax(180px,1fr)_120px_100px]"
                        >
                            <div
                                class="col-span-full sticky top-0 z-10 grid grid-cols-subgrid border-b border-eve-border bg-eve-bg-1 py-2 text-[10px] font-semibold tracking-wider text-eve-text-3"
                            >
                                <button
                                    class="text-left cursor-pointer hover:text-eve-text-1"
                                    @click="toggleSort('type_id')"
                                >
                                    TYPE ID{{ sortIndicator('type_id') }}
                                </button>
                                <button
                                    class="text-left cursor-pointer hover:text-eve-text-1"
                                    @click="toggleSort('name')"
                                >
                                    NAME{{ sortIndicator('name') }}
                                </button>
                                <button
                                    class="text-left cursor-pointer hover:text-eve-text-1"
                                    @click="toggleSort('type_name')"
                                >
                                    TYPE{{ sortIndicator('type_name') }}
                                </button>
                                <button
                                    class="text-left cursor-pointer hover:text-eve-text-1"
                                    @click="toggleSort('group_name')"
                                >
                                    GROUP{{ sortIndicator('group_name') }}
                                </button>
                                <button
                                    class="text-left cursor-pointer hover:text-eve-text-1"
                                    @click="toggleSort('category_name')"
                                >
                                    CATEGORY{{ sortIndicator('category_name') }}
                                </button>
                                <button
                                    class="text-left cursor-pointer hover:text-eve-text-1"
                                    @click="toggleSort('distance')"
                                >
                                    DISTANCE{{ sortIndicator('distance') }}
                                </button>
                            </div>

                            <div
                                v-for="(entry, index) in sortedEntries"
                                :key="`${index}:${entry.type_id ?? 'unknown'}:${entry.name}:${entry.distance ?? '-'}`"
                                class="col-span-full grid grid-cols-subgrid items-baseline py-2 text-sm"
                            >
                                <div class="font-mono text-eve-text-3">
                                    {{ entry.type_id ?? '—' }}
                                </div>
                                <div class="truncate text-eve-text-1">
                                    {{ entry.name }}
                                </div>
                                <div class="truncate text-eve-text-1">
                                    {{ entry.type_name }}
                                </div>
                                <div class="truncate text-eve-text-2">
                                    {{ entry.group_name ?? 'Unknown' }}
                                </div>
                                <div>
                                    <span
                                        class="text-xs"
                                        :class="
                                            entry.is_ship
                                                ? 'font-semibold text-eve-cyan'
                                                : 'text-eve-text-3'
                                        "
                                    >
                                        {{ entry.category_name ?? 'Unknown' }}
                                    </span>
                                </div>
                                <div class="font-mono text-eve-text-2">
                                    {{ entry.distance ?? '—' }}
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </section>
</template>
