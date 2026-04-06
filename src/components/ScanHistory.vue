<script setup lang="ts">
import { ref, watch } from 'vue'
import { useNow } from '@vueuse/core'
import { ChevronLeft, ChevronRight, Radar, Users } from 'lucide-vue-next'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
    activeNetworkId,
    fetchNetworkScans,
    latestSharedScan,
} from '../stores/intel'
import type { NetworkScan } from '../types'

const emit = defineEmits<{
    loadScan: [rawText: string]
}>()

const scans = ref<NetworkScan[]>([])
const currentPage = ref(1)
const lastPage = ref(1)
const loading = ref(false)

async function load(page: number = 1) {
    const networkId = activeNetworkId.value
    if (networkId == null) return
    loading.value = true
    try {
        const result = await fetchNetworkScans(networkId, page)
        scans.value = result.data
        currentPage.value = result.current_page
        lastPage.value = result.last_page
    } finally {
        loading.value = false
    }
}

// Load when network changes
watch(
    activeNetworkId,
    (id) => {
        if (id != null) {
            load()
        } else {
            scans.value = []
        }
    },
    { immediate: true }
)

// Prepend real-time scans
watch(latestSharedScan, (scan) => {
    if (scan && currentPage.value === 1) {
        scans.value = [scan, ...scans.value]
    }
})

function lineCount(rawText: string): number {
    return rawText.split('\n').filter((l) => l.trim()).length
}

const now = useNow({ interval: 30_000 })

function relativeTime(iso: string): string {
    const diff = now.value.getTime() - new Date(iso).getTime()
    const seconds = Math.floor(diff / 1000)
    if (seconds < 60) return 'just now'
    const minutes = Math.floor(seconds / 60)
    if (minutes < 60) return `${minutes}m ago`
    const hours = Math.floor(minutes / 60)
    if (hours < 24) return `${hours}h ago`
    const days = Math.floor(hours / 24)
    return `${days}d ago`
}
</script>

<template>
    <div
        v-if="scans.length === 0 && !loading"
        class="flex flex-col items-center justify-center py-12 text-center"
    >
        <div
            class="w-10 h-10 rounded-full bg-eve-bg-2 flex items-center justify-center mb-3"
        >
            <Radar class="w-5 h-5 text-eve-text-3" />
        </div>
        <p class="text-xs text-eve-text-3">No scans shared yet</p>
    </div>

    <template v-else>
        <div class="divide-y divide-eve-border/50">
            <button
                v-for="scan in scans"
                :key="scan.id"
                class="w-full flex items-center gap-3 px-4 py-2.5 hover:bg-eve-bg-1/50 transition-colors text-left cursor-pointer"
                @click="emit('loadScan', scan.raw_text)"
            >
                <div
                    class="w-7 h-7 rounded bg-eve-bg-2 flex items-center justify-center shrink-0"
                >
                    <Radar
                        v-if="scan.scan_type === 'dscan'"
                        class="w-3.5 h-3.5 text-eve-cyan"
                    />
                    <Users v-else class="w-3.5 h-3.5 text-eve-green" />
                </div>
                <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2">
                        <Badge
                            variant="outline"
                            class="text-[8px] h-4 px-1"
                            :class="
                                scan.scan_type === 'dscan'
                                    ? 'border-eve-cyan/40 text-eve-cyan'
                                    : 'border-eve-green/40 text-eve-green'
                            "
                            >{{
                                scan.scan_type === 'dscan' ? 'D-SCAN' : 'LOCAL'
                            }}</Badge
                        >
                        <span class="text-xs text-eve-text-1 truncate">
                            {{ scan.submitted_by?.character_name ?? 'Unknown' }}
                        </span>
                        <span class="text-[10px] text-eve-text-3"
                            >{{ lineCount(scan.raw_text) }} entries</span
                        >
                    </div>
                    <div class="flex items-center gap-2 mt-0.5">
                        <span
                            v-if="scan.solar_system"
                            class="text-[10px] text-eve-orange"
                            >{{ scan.solar_system }}</span
                        >
                        <span class="text-[10px] text-eve-text-3">{{
                            relativeTime(scan.created_at)
                        }}</span>
                    </div>
                </div>
            </button>
        </div>

        <!-- Pagination -->
        <div
            v-if="lastPage > 1"
            class="flex items-center justify-center gap-2 px-4 py-2 border-t border-eve-border"
        >
            <Button
                variant="ghost"
                size="sm"
                class="h-6 px-2 text-[10px]"
                :disabled="currentPage <= 1"
                @click="load(currentPage - 1)"
            >
                <ChevronLeft class="w-3 h-3" />
            </Button>
            <span class="text-[10px] text-eve-text-3"
                >{{ currentPage }} / {{ lastPage }}</span
            >
            <Button
                variant="ghost"
                size="sm"
                class="h-6 px-2 text-[10px]"
                :disabled="currentPage >= lastPage"
                @click="load(currentPage + 1)"
            >
                <ChevronRight class="w-3 h-3" />
            </Button>
        </div>
    </template>
</template>
