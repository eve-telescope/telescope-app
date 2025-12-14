<script setup lang="ts">
import { ExternalLink } from 'lucide-vue-next'
import { openUrl } from '@tauri-apps/plugin-opener'
import type { ZkillStats } from '../types'

const props = defineProps<{
    stats: ZkillStats
    characterId: number
}>()

function openZkill(): void {
    openUrl(`https://zkillboard.com/character/${props.characterId}/`)
}
</script>

<template>
    <div>
        <div class="space-y-0.5 text-xs">
            <div class="flex justify-between gap-4">
                <span class="text-eve-text-2">Solo Kills</span>
                <span class="font-mono text-eve-text-1">{{
                    stats.solo_kills.toLocaleString()
                }}</span>
            </div>
            <div class="flex justify-between gap-4">
                <span class="text-eve-text-2">Danger</span>
                <span class="font-mono text-eve-text-1"
                    >{{ stats.danger_ratio.toFixed(0) }}%</span
                >
            </div>
            <div class="flex justify-between gap-4">
                <span class="text-eve-text-2">Gang</span>
                <span class="font-mono text-eve-text-1"
                    >{{ stats.gang_ratio.toFixed(0) }}%</span
                >
            </div>
        </div>
        <button
            class="mt-3 text-[10px] text-eve-orange hover:text-eve-orange/80 transition-colors cursor-pointer flex items-center gap-1"
            @click.stop="openZkill()"
        >
            zKillboard
            <ExternalLink class="w-2.5 h-2.5" />
        </button>
    </div>
</template>
