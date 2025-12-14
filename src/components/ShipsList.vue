<script setup lang="ts">
import { openUrl } from '@tauri-apps/plugin-opener'
import type { ShipStats } from '../types'
import { getShipIconUrl } from '../utils/format'

const props = defineProps<{
    ships: ShipStats[]
    characterId: number
}>()

function openShipUrl(
    shipTypeId: number,
    type: 'kills' | 'losses' | 'all'
): void {
    const suffix = type === 'all' ? '' : `${type}/`
    openUrl(
        `https://zkillboard.com/character/${props.characterId}/ship/${shipTypeId}/${suffix}`
    )
}
</script>

<template>
    <div v-if="ships.length" class="space-y-1">
        <div
            v-for="ship in ships"
            :key="ship.ship_type_id"
            class="flex items-center gap-2 text-xs group relative"
        >
            <img
                :src="getShipIconUrl(ship.ship_type_id, 32)"
                class="w-5 h-5 rounded bg-eve-bg-3"
                loading="lazy"
            />
            <span class="truncate max-w-24">{{ ship.ship_name }}</span>
            <span class="font-mono text-eve-green text-[11px]">{{
                ship.kills
            }}</span>

            <!-- Hover links -->
            <div
                class="absolute right-0 opacity-0 group-hover:opacity-100 transition-opacity flex gap-0.5"
            >
                <button
                    class="px-1 py-0.5 rounded bg-eve-bg-3 text-eve-green text-[9px] font-mono hover:bg-eve-green/20 transition-colors"
                    @click.stop="openShipUrl(ship.ship_type_id, 'kills')"
                    title="View kills"
                >
                    K
                </button>
                <button
                    class="px-1 py-0.5 rounded bg-eve-bg-3 text-eve-red text-[9px] font-mono hover:bg-eve-red/20 transition-colors"
                    @click.stop="openShipUrl(ship.ship_type_id, 'losses')"
                    title="View losses"
                >
                    L
                </button>
                <button
                    class="px-1 py-0.5 rounded bg-eve-bg-3 text-eve-cyan text-[9px] font-mono hover:bg-eve-cyan/20 transition-colors"
                    @click.stop="openShipUrl(ship.ship_type_id, 'all')"
                    title="View all"
                >
                    B
                </button>
            </div>
        </div>
    </div>
    <div v-else class="text-xs text-eve-text-3 italic">No ship data</div>
</template>
