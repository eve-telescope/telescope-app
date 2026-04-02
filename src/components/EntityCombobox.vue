<script setup lang="ts">
import { ref } from 'vue'
import { ChevronsUpDown } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import {
    Command,
    CommandEmpty,
    CommandGroup,
    CommandInput,
    CommandItem,
    CommandList,
} from '@/components/ui/command'
import {
    Popover,
    PopoverContent,
    PopoverTrigger,
} from '@/components/ui/popover'
import { searchEntities } from '../stores/intel'
import type { EntityType, SearchResult } from '../types'

const model = defineModel<{
    id: string
    name: string
    type: EntityType | null
}>({ required: true })

const props = withDefaults(
    defineProps<{
        placeholder?: string
        category?: EntityType
    }>(),
    {
        placeholder: 'Search entity...',
    }
)

const open = ref(false)
const query = ref('')
const results = ref<SearchResult[]>([])

async function handleSearch(value: string | number) {
    query.value = String(value)
    if (query.value.length < 2) return
    try {
        results.value = await searchEntities(query.value, props.category)
    } catch {
        results.value = []
    }
}

function select(entity: SearchResult) {
    model.value = {
        id: String(entity.id),
        name: entity.name,
        type: entity.category as EntityType,
    }
    query.value = ''
    results.value = []
    open.value = false
}

function getPortraitUrl(type: string, id: number, size = 32): string {
    if (type === 'character')
        return `https://images.evetech.net/characters/${id}/portrait?size=${size}`
    if (type === 'corporation')
        return `https://images.evetech.net/corporations/${id}/logo?size=${size}`
    if (type === 'alliance')
        return `https://images.evetech.net/alliances/${id}/logo?size=${size}`
    return ''
}
</script>

<template>
    <Popover v-model:open="open">
        <PopoverTrigger as-child>
            <Button
                variant="outline"
                role="combobox"
                :aria-expanded="open"
                class="w-full justify-between bg-eve-bg-2 border-eve-border text-xs hover:bg-eve-bg-3"
            >
                <div v-if="model.id" class="flex items-center gap-2 min-w-0">
                    <img
                        :src="getPortraitUrl(model.type!, parseInt(model.id))"
                        class="w-5 h-5 rounded shrink-0"
                    />
                    <span class="truncate">{{ model.name }}</span>
                    <span
                        v-if="model.type"
                        class="text-[9px] uppercase text-eve-text-3"
                        >{{ model.type }}</span
                    >
                </div>
                <span v-else class="text-eve-text-3">{{ placeholder }}</span>
                <ChevronsUpDown class="ml-2 h-3.5 w-3.5 shrink-0 opacity-50" />
            </Button>
        </PopoverTrigger>
        <PopoverContent
            class="p-0 bg-eve-bg-2 border-eve-border w-[var(--reka-popover-trigger-width)]"
            align="start"
        >
            <Command class="bg-eve-bg-2" :filter-function="() => 1">
                <CommandInput
                    :placeholder="placeholder"
                    @update:model-value="handleSearch"
                />
                <CommandList class="max-h-48">
                    <CommandEmpty>No matching entities</CommandEmpty>
                    <CommandGroup>
                        <CommandItem
                            v-for="result in results"
                            :key="`${result.category}:${result.id}`"
                            :value="`${result.category}:${result.id}`"
                            class="gap-2 px-3 py-2"
                            @select="select(result)"
                        >
                            <img
                                :src="
                                    getPortraitUrl(result.category, result.id)
                                "
                                class="w-6 h-6 rounded shrink-0"
                            />
                            <div class="min-w-0">
                                <div class="text-xs text-eve-text-1 truncate">
                                    {{ result.name }}
                                    <span
                                        class="ml-1 uppercase text-[9px] text-eve-cyan"
                                        >{{ result.category }}</span
                                    >
                                    <span
                                        v-if="result.ticker"
                                        class="text-eve-text-3"
                                        >[{{ result.ticker }}]</span
                                    >
                                </div>
                                <div
                                    v-if="result.corporation || result.alliance"
                                    class="text-[9px] text-eve-text-3 truncate"
                                >
                                    <span v-if="result.corporation"
                                        >[{{ result.corporation.ticker }}]
                                        {{ result.corporation.name }}</span
                                    >
                                    <span
                                        v-if="
                                            result.corporation &&
                                            result.alliance
                                        "
                                    >
                                        ·
                                    </span>
                                    <span v-if="result.alliance"
                                        >&lt;{{ result.alliance.ticker }}&gt;
                                        {{ result.alliance.name }}</span
                                    >
                                </div>
                            </div>
                        </CommandItem>
                    </CommandGroup>
                </CommandList>
            </Command>
        </PopoverContent>
    </Popover>
</template>
