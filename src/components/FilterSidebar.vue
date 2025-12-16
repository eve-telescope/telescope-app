<script setup lang="ts">
import { computed, toRef } from 'vue'
import { X } from 'lucide-vue-next'
import type { PilotIntel } from '../types'
import FilterGroup from './FilterGroup.vue'
import { usePilotCounts } from '../composables/usePilotCounts'

const props = defineProps<{
    pilots: PilotIntel[]
    selectedCorps: Set<string>
    selectedAlliances: Set<string>
    selectedTags: Set<string>
}>()

const emit = defineEmits<{
    toggleCorp: [name: string]
    toggleAlliance: [name: string]
    toggleTag: [tag: string]
    clearFilters: []
}>()

const { tagCounts } = usePilotCounts(toRef(props, 'pilots'))

interface GroupInfo {
    id: number
    name: string
    ticker: string
    count: number
}

const corporations = computed<GroupInfo[]>(() => {
    const groups = new Map<
        string,
        { id: number; ticker: string; count: number }
    >()
    for (const p of props.pilots) {
        const name = p.character.corporation_name || 'Unknown'
        const id = p.character.corporation_id || 0
        const ticker = p.character.corporation_ticker || ''
        const existing = groups.get(name)
        if (existing) {
            existing.count++
        } else {
            groups.set(name, { id, ticker, count: 1 })
        }
    }
    return Array.from(groups.entries())
        .map(([name, { id, ticker, count }]) => ({ id, name, ticker, count }))
        .sort((a, b) => b.count - a.count)
})

const alliances = computed<GroupInfo[]>(() => {
    const groups = new Map<
        string,
        { id: number; ticker: string; count: number }
    >()
    for (const p of props.pilots) {
        const name = p.character.alliance_name
        const id = p.character.alliance_id
        const ticker = p.character.alliance_ticker || ''
        if (name && id) {
            const existing = groups.get(name)
            if (existing) {
                existing.count++
            } else {
                groups.set(name, { id, ticker, count: 1 })
            }
        }
    }
    return Array.from(groups.entries())
        .map(([name, { id, ticker, count }]) => ({ id, name, ticker, count }))
        .sort((a, b) => b.count - a.count)
})

interface TagInfo {
    key: string
    label: string
    count: number
    colorClass: string
}

const TAG_DEFS: {
    key: keyof typeof tagCounts.value
    label: string
    colorClass: string
}[] = [
    {
        key: 'super',
        label: 'SUPER',
        colorClass: 'bg-rose-500/20 text-rose-400 border-rose-500/30',
    },
    {
        key: 'capital',
        label: 'CAPITAL',
        colorClass: 'bg-amber-500/20 text-amber-400 border-amber-500/30',
    },
    {
        key: 'blops',
        label: 'BLACK OPS',
        colorClass: 'bg-indigo-500/20 text-indigo-400 border-indigo-500/30',
    },
    {
        key: 'recon',
        label: 'RECON',
        colorClass: 'bg-teal-500/20 text-teal-400 border-teal-500/30',
    },
    {
        key: 'cyno',
        label: 'CYNO',
        colorClass: 'bg-purple-500/20 text-purple-400 border-purple-500/30',
    },
    {
        key: 'solo',
        label: 'SOLO',
        colorClass: 'bg-sky-500/20 text-sky-400 border-sky-500/30',
    },
]

const tags = computed<TagInfo[]>(() => {
    return TAG_DEFS.filter((t) => tagCounts.value[t.key] > 0).map((t) => ({
        ...t,
        count: tagCounts.value[t.key],
    }))
})

const hasFilters = computed(() => {
    return (
        props.selectedCorps.size > 0 ||
        props.selectedAlliances.size > 0 ||
        props.selectedTags.size > 0
    )
})

function getCorpLogo(id: number): string {
    return `https://images.evetech.net/corporations/${id}/logo?size=32`
}

function getAllianceLogo(id: number): string {
    return `https://images.evetech.net/alliances/${id}/logo?size=32`
}
</script>

<template>
    <aside
        class="w-64 bg-eve-bg-1 border-l border-eve-border flex flex-col overflow-hidden"
    >
        <div
            class="flex justify-between items-center px-4 py-3 border-b border-eve-border shrink-0"
        >
            <span
                class="text-[10px] font-semibold tracking-[0.15em] text-eve-text-3"
                >FILTERS</span
            >
            <button
                class="w-5 h-5 flex items-center justify-center rounded text-eve-text-3 cursor-pointer transition-all hover:bg-eve-bg-hover hover:text-eve-text-1"
                :class="
                    hasFilters ? 'opacity-100' : 'opacity-0 pointer-events-none'
                "
                @click="emit('clearFilters')"
                title="Clear filters"
            >
                <X class="w-3 h-3" />
            </button>
        </div>

        <!-- Scrollable filter lists -->
        <div class="flex-1 overflow-y-auto min-h-0">
            <!-- Tags -->
            <div v-if="tags.length > 0" class="p-3 border-b border-eve-border">
                <h4
                    class="text-[10px] font-semibold tracking-wider text-eve-text-3 uppercase mb-2"
                >
                    Tags
                </h4>
                <div class="flex flex-wrap gap-1.5">
                    <button
                        v-for="tag in tags"
                        :key="tag.key"
                        class="flex items-center gap-1.5 px-2 py-1 text-[10px] font-bold rounded border transition-colors"
                        :class="[
                            tag.colorClass,
                            selectedTags.has(tag.key)
                                ? 'ring-1 ring-white/30'
                                : 'opacity-70 hover:opacity-100',
                        ]"
                        @click="emit('toggleTag', tag.key)"
                    >
                        {{ tag.label }}
                        <span class="font-mono text-[9px] opacity-75">{{
                            tag.count
                        }}</span>
                    </button>
                </div>
            </div>

            <!-- Alliances -->
            <div
                v-if="alliances.length > 0"
                class="p-3 border-b border-eve-border"
            >
                <FilterGroup
                    title="Alliances"
                    :items="alliances"
                    :selected="selectedAlliances"
                    :get-logo-url="getAllianceLogo"
                    @toggle="emit('toggleAlliance', $event)"
                />
            </div>

            <!-- Corporations -->
            <div class="p-3">
                <FilterGroup
                    title="Corporations"
                    :items="corporations"
                    :selected="selectedCorps"
                    :get-logo-url="getCorpLogo"
                    @toggle="emit('toggleCorp', $event)"
                />
            </div>
        </div>
    </aside>
</template>
