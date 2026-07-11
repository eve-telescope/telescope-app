<script setup lang="ts">
import { Shield, Trash2 } from 'lucide-vue-next'
import { Badge } from '@/components/ui/badge'
import type { NetworkAccess } from '../../types'
import {
    accessPermissionLabel,
    canRemoveAccess,
    getPortraitUrl,
} from '../../utils/network'

defineProps<{
    accesses: NetworkAccess[]
}>()

const emit = defineEmits<{
    delete: [accessId: number]
}>()
</script>

<template>
    <div class="divide-y divide-eve-border/50">
        <div
            v-for="access in accesses"
            :key="access.id"
            class="group flex items-center gap-3 px-4 py-2.5 hover:bg-eve-bg-1/50 transition-colors"
        >
            <img
                v-if="access.entity"
                :src="
                    getPortraitUrl(access.accessible_type, access.accessible_id)
                "
                class="w-7 h-7 rounded shrink-0"
            />
            <div
                v-else
                class="w-7 h-7 rounded bg-eve-bg-2 flex items-center justify-center shrink-0"
            >
                <Shield class="w-3.5 h-3.5 text-eve-text-3" />
            </div>

            <div class="flex-1 min-w-0">
                <div class="flex items-center gap-1.5">
                    <span class="text-xs font-medium text-eve-text-1 truncate">
                        {{ access.entity?.name ?? `#${access.accessible_id}` }}
                    </span>
                    <span
                        v-if="access.entity?.ticker"
                        class="text-[10px] text-eve-text-3"
                        >[{{ access.entity.ticker }}]</span
                    >
                </div>
                <div
                    v-if="access.entity?.corporation || access.entity?.alliance"
                    class="text-[10px] text-eve-text-3 mt-0.5 truncate"
                >
                    <span v-if="access.entity?.corporation"
                        >[{{ access.entity.corporation.ticker }}]
                        {{ access.entity.corporation.name }}</span
                    >
                    <span
                        v-if="
                            access.entity?.corporation &&
                            access.entity?.alliance
                        "
                    >
                        ·
                    </span>
                    <span v-if="access.entity?.alliance"
                        >&lt;{{ access.entity.alliance.ticker }}&gt;
                        {{ access.entity.alliance.name }}</span
                    >
                </div>
            </div>

            <div class="flex items-center gap-2 shrink-0">
                <Badge
                    variant="outline"
                    class="text-[9px] h-5"
                    :class="
                        access.is_owner
                            ? 'border-eve-cyan/40 text-eve-cyan'
                            : ''
                    "
                    >{{ accessPermissionLabel(access) }}</Badge
                >
                <button
                    v-if="canRemoveAccess(access)"
                    class="p-1 rounded text-eve-text-3 opacity-0 group-hover:opacity-100 hover:text-eve-red hover:bg-eve-red/10 transition-all"
                    @click="emit('delete', access.id)"
                >
                    <Trash2 class="w-3 h-3" />
                </button>
            </div>
        </div>
    </div>
</template>
