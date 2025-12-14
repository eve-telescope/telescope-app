<script setup lang="ts">
import { X, Loader2, Trash2, HelpCircle } from 'lucide-vue-next'
import { invoke } from '@tauri-apps/api/core'
import type { PilotIntel } from '../types'
import type { LookupProgress } from '../composables/usePilots'
import ThreatSummary from './ThreatSummary.vue'
import ShortcutEditor from './ShortcutEditor.vue'
import LookupProgressBar from './LookupProgress.vue'
import ShareButton from './ShareButton.vue'
import { openAboutWindow } from '../composables/useAboutWindow'

defineProps<{
    loading: boolean
    pilotCount: number
    pilots: PilotIntel[]
    progress: LookupProgress | null
    shortcut: string
}>()

const pilotNames = defineModel<string>('pilotNames', { required: true })

const emit = defineEmits<{
    scan: []
    clear: []
    'update:shortcut': [shortcut: string]
}>()

function handleKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
        emit('scan')
    }
}

async function clearCache() {
    try {
        await invoke('clear_cache')
    } catch (e) {
        console.error('Failed to clear cache:', e)
    }
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
                PASTE LOCAL
            </h3>
            <textarea
                v-model="pilotNames"
                placeholder="Pilot names..."
                :disabled="loading"
                @keydown="handleKeydown"
                class="w-full flex-1 min-h-32 px-2.5 py-2 bg-eve-bg-2 border border-eve-border rounded text-eve-text-1 font-mono text-[11px] leading-relaxed resize-none focus:outline-none focus:border-eve-cyan-dim placeholder:text-eve-text-3 disabled:opacity-50"
            ></textarea>

            <div class="flex gap-2 mt-2 shrink-0">
                <button
                    class="flex-1 py-2 bg-eve-cyan rounded text-eve-bg-0 text-xs font-bold tracking-wider cursor-pointer transition-all hover:bg-eve-cyan-dim disabled:opacity-40 disabled:cursor-not-allowed flex items-center justify-center gap-2"
                    :disabled="loading || !pilotNames.trim()"
                    @click="emit('scan')"
                >
                    <Loader2 v-if="loading" class="w-3.5 h-3.5 animate-spin" />
                    <template v-else>
                        SCAN
                        <span
                            v-if="pilotCount > 0"
                            class="bg-black/20 px-1.5 py-0.5 rounded text-[10px]"
                            >{{ pilotCount }}</span
                        >
                    </template>
                </button>

                <button
                    v-if="pilotNames.trim()"
                    class="px-3 py-2 bg-transparent border border-eve-border rounded text-eve-text-3 cursor-pointer transition-colors hover:border-eve-text-3 hover:text-eve-text-1 flex items-center justify-center"
                    @click="emit('clear')"
                    title="Clear"
                >
                    <X class="w-3.5 h-3.5" />
                </button>
            </div>

            <!-- Progress Bar -->
            <LookupProgressBar
                v-if="loading && progress"
                :progress="progress"
                class="mt-3 shrink-0"
            />
        </div>

        <!-- Threat Summary -->
        <div v-if="pilots.length > 0" class="p-3 shrink-0">
            <h3
                class="text-[10px] font-semibold tracking-[0.15em] text-eve-text-3 mb-2"
            >
                THREAT SUMMARY
            </h3>
            <ThreatSummary :pilots="pilots" />
        </div>

        <!-- Share Button -->
        <div v-if="pilots.length > 0" class="px-3 pb-3 shrink-0">
            <ShareButton :pilot-names="pilotNames" />
        </div>

        <!-- Shortcut Settings -->
        <div class="p-3 border-t border-eve-border shrink-0">
            <ShortcutEditor
                :shortcut="shortcut"
                @update="emit('update:shortcut', $event)"
            />
            <div class="flex items-center justify-between mt-3">
                <p class="text-[9px] text-eve-text-3">Data from zKillboard</p>
                <div class="flex items-center gap-1">
                    <button
                        class="p-1.5 rounded text-eve-text-3 hover:text-eve-cyan hover:bg-eve-cyan/10 transition-all"
                        @click="openAboutWindow"
                        title="About & Help"
                    >
                        <HelpCircle class="w-4 h-4" />
                    </button>
                    <button
                        class="p-1.5 rounded text-eve-text-3 hover:text-eve-orange hover:bg-eve-orange/10 transition-all"
                        @click="clearCache"
                        title="Clear cache"
                    >
                        <Trash2 class="w-4 h-4" />
                    </button>
                </div>
            </div>
        </div>
    </aside>
</template>
