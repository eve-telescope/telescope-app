<script setup lang="ts">
import { Plus, Trash2, Network, LogOut, Zap, RefreshCw } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import type { IntelNetwork } from '../../types'

const newNetworkName = defineModel<string>({ required: true })

defineProps<{
    networks: IntelNetwork[]
    activeNetworkId: number | null
    loading: boolean
    togglingId: number | null
}>()

const emit = defineEmits<{
    create: []
    select: [networkId: number]
    toggleConnect: [networkId: number]
    delete: [networkId: number]
}>()
</script>

<template>
    <div class="p-4 space-y-3">
        <div class="flex gap-2">
            <Input
                v-model="newNetworkName"
                type="text"
                placeholder="New network name..."
                class="flex-1"
                @keydown.enter="emit('create')"
            />
            <Button
                size="sm"
                :disabled="!newNetworkName.trim() || loading"
                @click="emit('create')"
            >
                <Plus class="w-3.5 h-3.5" />
                Create
            </Button>
        </div>

        <div
            v-if="networks.length === 0"
            class="flex flex-col items-center py-12 text-center"
        >
            <div
                class="w-10 h-10 rounded-full bg-eve-bg-2 flex items-center justify-center mb-3"
            >
                <Network class="w-5 h-5 text-eve-text-3" />
            </div>
            <p class="text-xs text-eve-text-3">
                No networks yet. Create one to get started.
            </p>
        </div>

        <div
            v-for="network in networks"
            :key="network.id"
            class="group flex items-center gap-3 p-3 rounded-lg border transition-colors cursor-pointer"
            :class="
                activeNetworkId === network.id
                    ? 'border-eve-cyan/30 bg-eve-cyan/5'
                    : 'border-eve-border bg-eve-bg-1 hover:border-eve-text-3/30 hover:bg-eve-bg-2'
            "
            @click="emit('select', network.id)"
        >
            <div
                class="w-8 h-8 rounded bg-eve-bg-2 flex items-center justify-center shrink-0"
            >
                <Network
                    class="w-4 h-4"
                    :class="
                        activeNetworkId === network.id
                            ? 'text-eve-cyan'
                            : 'text-eve-text-3'
                    "
                />
            </div>
            <div class="flex-1 min-w-0">
                <div class="text-xs font-medium text-eve-text-1 truncate">
                    {{ network.name }}
                </div>
                <div class="text-[10px] text-eve-text-3 mt-0.5">
                    {{ network.entries_count ?? 0 }} annotations
                </div>
            </div>
            <div class="flex items-center gap-2 shrink-0">
                <span
                    class="flex items-center gap-1.5 text-[10px] font-medium"
                    :class="
                        activeNetworkId === network.id
                            ? 'text-eve-cyan'
                            : 'text-eve-text-3'
                    "
                >
                    <span
                        class="w-1.5 h-1.5 rounded-full"
                        :class="
                            activeNetworkId === network.id
                                ? 'bg-eve-cyan shadow-[0_0_6px] shadow-eve-cyan'
                                : 'bg-eve-text-3/40'
                        "
                    />
                    {{
                        activeNetworkId === network.id ? 'Connected' : 'Offline'
                    }}
                </span>

                <Button
                    v-if="activeNetworkId === network.id"
                    variant="ghost"
                    size="sm"
                    class="h-7 px-2 text-[10px] text-eve-text-3 hover:text-eve-text-1"
                    :disabled="togglingId === network.id"
                    @click.stop="emit('toggleConnect', network.id)"
                >
                    <RefreshCw
                        v-if="togglingId === network.id"
                        class="w-3.5 h-3.5 mr-1 animate-spin"
                    />
                    <LogOut v-else class="w-3.5 h-3.5 mr-1" />
                    Disconnect
                </Button>
                <Button
                    v-else
                    size="sm"
                    class="h-7 px-3 text-[10px]"
                    :disabled="togglingId === network.id"
                    @click.stop="emit('toggleConnect', network.id)"
                >
                    <RefreshCw
                        v-if="togglingId === network.id"
                        class="w-3.5 h-3.5 mr-1 animate-spin"
                    />
                    <Zap v-else class="w-3.5 h-3.5 mr-1" />
                    Connect
                </Button>

                <button
                    class="p-1.5 rounded text-eve-text-3 opacity-0 group-hover:opacity-100 hover:text-eve-red hover:bg-eve-red/10 transition-all shrink-0"
                    title="Delete network"
                    @click.stop="emit('delete', network.id)"
                >
                    <Trash2 class="w-3 h-3" />
                </button>
            </div>
        </div>
    </div>
</template>
