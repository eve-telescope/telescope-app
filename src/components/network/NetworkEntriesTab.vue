<script setup lang="ts">
import { Plus, Trash2, Pencil, Zap } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import type { IntelAnnotation } from '../../types'
import {
    DEFAULT_TAG_COLOR,
    DEFAULT_TAG_TEXT_COLOR,
} from '../../utils/pilotTags'

defineProps<{
    annotations: IntelAnnotation[]
}>()

const emit = defineEmits<{
    add: []
    edit: [entry: IntelAnnotation]
    delete: [entryId: number]
}>()
</script>

<template>
    <div
        v-if="annotations.length === 0"
        class="flex flex-col items-center justify-center py-12 text-center"
    >
        <div
            class="w-10 h-10 rounded-full bg-eve-bg-2 flex items-center justify-center mb-3"
        >
            <Zap class="w-5 h-5 text-eve-text-3" />
        </div>
        <p class="text-xs text-eve-text-3">No annotations yet</p>
        <Button
            variant="ghost"
            size="sm"
            class="mt-2 text-[10px] text-eve-cyan hover:text-eve-cyan"
            @click="emit('add')"
        >
            <Plus class="w-3 h-3 mr-1" />
            Add your first annotation
        </Button>
    </div>

    <div v-else class="divide-y divide-eve-border/50">
        <div
            v-for="entry in annotations"
            :key="entry.id"
            v-memo="[entry]"
            class="group flex items-start gap-3 px-4 py-2.5 hover:bg-eve-bg-1/50 transition-colors"
        >
            <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2">
                    <span
                        class="text-xs font-medium text-eve-text-1 truncate"
                        >{{ entry.targetName }}</span
                    >
                    <span class="text-[9px] uppercase text-eve-text-3">{{
                        entry.targetType
                    }}</span>
                </div>
                <div class="flex flex-wrap gap-1 mt-1">
                    <Badge
                        v-for="tag in entry.tags"
                        :key="`${entry.id}:${tag}`"
                        variant="secondary"
                        class="text-[9px] h-4 px-1.5"
                        :style="{
                            backgroundColor:
                                (entry.color || DEFAULT_TAG_COLOR) + '22',
                            color: entry.color || DEFAULT_TAG_TEXT_COLOR,
                        }"
                        >{{ tag }}</Badge
                    >
                </div>
                <p
                    v-if="entry.note"
                    class="text-[10px] text-eve-text-3 mt-1 truncate"
                >
                    {{ entry.note }}
                </p>
            </div>
            <div
                class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity shrink-0 pt-0.5"
            >
                <button
                    class="p-1 rounded text-eve-text-3 hover:text-eve-cyan hover:bg-eve-cyan/10 transition-colors"
                    @click="emit('edit', entry)"
                >
                    <Pencil class="w-3 h-3" />
                </button>
                <button
                    class="p-1 rounded text-eve-text-3 hover:text-eve-red hover:bg-eve-red/10 transition-colors"
                    @click="emit('delete', entry.id)"
                >
                    <Trash2 class="w-3 h-3" />
                </button>
            </div>
        </div>
    </div>
</template>
