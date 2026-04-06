<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { useNow } from '@vueuse/core'
import { X, Loader2, Radar, Users, RefreshCw } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import type { PilotIntel, NetworkScan } from '../types'
import type { LookupProgress } from '../composables/usePilots'
import {
    isAuthenticated,
    activeNetworkId,
    activeNetwork,
    fetchNetworkScans,
    latestSharedScan,
} from '../stores/intel'
import ThreatSummary from './ThreatSummary.vue'
import LookupProgressBar from './LookupProgress.vue'
import ShareButton from './ShareButton.vue'

const props = defineProps<{
    loading: boolean
    pilotCount: number
    pilots: PilotIntel[]
    progress: LookupProgress | null
}>()

const pilotNames = defineModel<string>('pilotNames', { required: true })

const emit = defineEmits<{
    scan: []
    clear: []
    loadScan: [rawText: string]
}>()

// Delay progress bar to avoid flashing on fast scans
const showProgress = ref(false)
let progressTimer: ReturnType<typeof setTimeout> | null = null
watch(
    () => props.loading,
    (loading) => {
        if (loading) {
            progressTimer = setTimeout(() => {
                showProgress.value = true
            }, 500)
        } else {
            if (progressTimer) clearTimeout(progressTimer)
            progressTimer = null
            showProgress.value = false
        }
    }
)

// Keep last non-empty pilots for stable threat summary during rescans
const lastPilots = ref<PilotIntel[]>([])
const summaryPilots = computed(() =>
    props.pilots.length > 0 ? props.pilots : lastPilots.value
)
watch(
    () => props.pilots,
    (p) => {
        if (p.length > 0) lastPilots.value = p
    }
)

function handleKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
        emit('scan')
    }
}

// Scan history
const recentScans = ref<NetworkScan[]>([])
const refreshingScans = ref(false)

async function loadScans() {
    const id = activeNetworkId.value
    if (!id || !isAuthenticated.value) {
        recentScans.value = []
        return
    }
    refreshingScans.value = true
    try {
        const result = await fetchNetworkScans(id, 1)
        recentScans.value = result.data.slice(0, 10)
    } catch {
        recentScans.value = []
    } finally {
        refreshingScans.value = false
    }
}

watch([activeNetworkId, isAuthenticated], () => loadScans(), {
    immediate: true,
})

watch(latestSharedScan, (scan) => {
    if (scan) {
        recentScans.value = [scan, ...recentScans.value].slice(0, 10)
    }
})

const now = useNow({ interval: 30_000 })

function relativeTime(iso: string): string {
    const diff = now.value.getTime() - new Date(iso).getTime()
    const seconds = Math.floor(diff / 1000)
    if (seconds < 60) return 'now'
    const minutes = Math.floor(seconds / 60)
    if (minutes < 60) return `${minutes}m`
    const hours = Math.floor(minutes / 60)
    if (hours < 24) return `${hours}h`
    return `${Math.floor(hours / 24)}d`
}

function lineCount(text: string): number {
    return text.split('\n').filter((l) => l.trim()).length
}
</script>

<template>
    <aside
        class="w-64 bg-eve-bg-1 border-r border-eve-border flex flex-col shrink-0"
    >
        <!-- Input Section -->
        <div
            class="p-3 border-b border-eve-border flex-1 flex flex-col min-h-0"
        >
            <h3
                class="text-[10px] font-semibold tracking-[0.15em] text-eve-text-3 mb-2 shrink-0"
            >
                PASTE LOCAL OR D-SCAN
            </h3>
            <Textarea
                v-model="pilotNames"
                placeholder="Paste local or D-scan text..."
                :disabled="loading"
                @keydown="handleKeydown"
                class="w-full flex-1 min-h-32 resize-none font-mono"
            />
            <p class="mt-2 text-[9px] text-eve-text-3 shrink-0">
                Telescope will infer the input type from the pasted message
                format.
            </p>

            <div class="flex gap-2 mt-2 shrink-0">
                <Button
                    class="flex-1"
                    :disabled="loading || !pilotNames.trim()"
                    @click="emit('scan')"
                >
                    <Loader2 v-if="loading" class="w-3.5 h-3.5 animate-spin" />
                    <template v-else>
                        SCAN
                        <span
                            v-if="pilotCount > 0"
                            class="rounded bg-black/20 px-1.5 py-0.5 text-[10px]"
                            >{{ pilotCount }}</span
                        >
                    </template>
                </Button>

                <Button
                    variant="outline"
                    size="icon"
                    :disabled="!pilotNames.trim()"
                    @click="emit('clear')"
                    title="Clear"
                >
                    <X class="w-3.5 h-3.5" />
                </Button>
            </div>

            <!-- Progress Bar -->
            <LookupProgressBar
                v-if="showProgress && progress"
                :progress="progress"
                class="mt-3 shrink-0"
            />
        </div>

        <!-- Threat Summary -->
        <div class="p-3 border-b border-eve-border shrink-0">
            <h3
                class="text-[10px] font-semibold tracking-[0.15em] text-eve-text-3 mb-2"
            >
                THREAT SUMMARY
            </h3>
            <ThreatSummary
                v-if="summaryPilots.length > 0"
                :pilots="summaryPilots"
            />
            <p v-else class="text-[9px] text-eve-text-3">No scan results yet</p>
        </div>

        <!-- Share Button -->
        <div class="px-3 py-3 border-b border-eve-border shrink-0">
            <ShareButton
                :pilot-names="pilotNames"
                :disabled="summaryPilots.length === 0"
            />
        </div>

        <!-- Recent Scans -->
        <div
            v-if="isAuthenticated && activeNetwork && recentScans.length > 0"
            class="flex flex-col h-52 overflow-hidden shrink-0"
        >
            <div class="flex items-center justify-between px-3 pt-3 pb-1.5 shrink-0">
                <h3 class="text-[10px] font-semibold tracking-[0.15em] text-eve-text-3">
                    RECENT SCANS
                </h3>
                <button
                    class="p-0.5 rounded text-eve-text-3 hover:text-eve-text-1 hover:bg-white/5 transition-colors disabled:opacity-50"
                    :disabled="refreshingScans"
                    title="Refresh scans"
                    @click="loadScans"
                >
                    <RefreshCw
                        class="w-3 h-3"
                        :class="refreshingScans ? 'animate-spin' : ''"
                    />
                </button>
            </div>
            <div class="flex-1 overflow-y-auto">
                <button
                    v-for="scan in recentScans"
                    :key="scan.id"
                    class="w-full flex items-center gap-2 px-3 py-1.5 hover:bg-eve-bg-hover transition-colors text-left cursor-pointer"
                    @click="emit('loadScan', scan.raw_text)"
                >
                    <Radar
                        v-if="scan.scan_type === 'dscan'"
                        class="w-3 h-3 text-eve-cyan shrink-0"
                    />
                    <Users v-else class="w-3 h-3 text-eve-green shrink-0" />
                    <div class="flex-1 min-w-0">
                        <div class="text-[10px] text-eve-text-1 truncate">
                            {{ scan.submitted_by?.character_name ?? 'Unknown' }}
                        </div>
                    </div>
                    <span
                        class="text-[9px] text-eve-text-3 tabular-nums shrink-0"
                        >{{ lineCount(scan.raw_text) }}</span
                    >
                    <span
                        class="text-[9px] text-eve-text-3 tabular-nums shrink-0 w-6 text-right"
                        >{{ relativeTime(scan.created_at) }}</span
                    >
                </button>
            </div>
        </div>
    </aside>
</template>
