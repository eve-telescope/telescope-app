<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { listen, emit, type UnlistenFn } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { openUrl } from '@tauri-apps/plugin-opener'
import type { PilotIntel } from '../types'
import { getPortraitUrl, getShipIconUrl, getKdRatio } from '../utils/format'
import { getFlagLabels } from '../utils/intel'
import { usePilotCounts } from '../composables/usePilotCounts'
import { useSyncedFilters } from '../composables/useSyncedFilters'
import { usePilotSort } from '../composables/usePilotSort'
import { useSettings } from '../composables/useSettings'

function openZkill(characterId: number) {
    if (characterId > 0) {
        openUrl(`https://zkillboard.com/character/${characterId}/`)
    }
}

function openShipZkill(characterId: number, shipTypeId: number) {
    if (characterId > 0 && shipTypeId > 0) {
        openUrl(
            `https://zkillboard.com/character/${characterId}/ship/${shipTypeId}/`
        )
    }
}

const pilots = ref<PilotIntel[]>([])

const { settings } = useSettings()
const locked = computed({
    get: () => settings.value.overlayLocked,
    set: (val) => {
        settings.value.overlayLocked = val
    },
})

const { threatCounts, tagCounts } = usePilotCounts(pilots)
const {
    threatFilter,
    selectedTags,
    filteredPilots,
    toggleThreatFilter,
    toggleTag,
} = useSyncedFilters(pilots)

const { sortKey, sortDirection, handleSort, sortedPilots } =
    usePilotSort(filteredPilots)

let unlistenSync: UnlistenFn | null = null
let unlistenClear: UnlistenFn | null = null

onMounted(async () => {
    unlistenSync = await listen<PilotIntel[]>('pilots-sync', (event) => {
        pilots.value = event.payload
    })

    unlistenClear = await listen('overlay-clear', () => {
        pilots.value = []
    })

    // Apply persisted lock state
    if (locked.value) {
        try {
            const window = getCurrentWindow()
            await window.setResizable(false)
        } catch (e) {
            console.error('Failed to apply lock state:', e)
        }
    }

    emit('overlay-sync-request')
})

onUnmounted(() => {
    unlistenSync?.()
    unlistenClear?.()
})

async function closeOverlay() {
    await invoke('close_overlay')
}

async function toggleLock() {
    const newLocked = !locked.value
    const window = getCurrentWindow()
    try {
        await window.setResizable(!newLocked)
        locked.value = newLocked
    } catch (e) {
        console.error('Failed to set resizable:', e)
        // Still update the lock state for drag prevention even if resize fails
        locked.value = newLocked
    }
}
</script>

<template>
    <div
        class="h-screen w-screen select-none overflow-hidden flex flex-col bg-eve-bg-0"
    >
        <!-- Header -->
        <div
            class="flex items-center justify-between px-3 py-1.5 bg-eve-bg-1 border-b border-eve-cyan/30 shrink-0"
            :class="locked ? 'cursor-default' : 'cursor-move'"
            :data-tauri-drag-region="!locked || undefined"
        >
            <div class="flex items-center gap-2">
                <div
                    class="w-2 h-2 rounded-full bg-eve-cyan animate-pulse"
                ></div>
                <span class="text-xs font-bold text-eve-cyan">{{
                    pilots.length
                }}</span>
            </div>

            <div class="flex items-center gap-1">
                <!-- Lock button -->
                <button
                    class="w-6 h-6 flex items-center justify-center rounded transition-colors"
                    :class="
                        locked
                            ? 'text-eve-cyan bg-eve-cyan/20'
                            : 'text-eve-text-3 hover:text-eve-text-1 hover:bg-eve-bg-3'
                    "
                    @click="toggleLock"
                    :title="locked ? 'Unlock position' : 'Lock position'"
                >
                    <svg
                        v-if="locked"
                        class="w-3.5 h-3.5"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"
                        />
                    </svg>
                    <svg
                        v-else
                        class="w-3.5 h-3.5"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M8 11V7a4 4 0 118 0m-4 8v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2z"
                        />
                    </svg>
                </button>

                <!-- Close button -->
                <button
                    class="w-6 h-6 flex items-center justify-center text-eve-text-3 hover:text-eve-red rounded hover:bg-eve-bg-3 transition-colors"
                    @click="closeOverlay"
                    title="Close overlay"
                >
                    <svg
                        class="w-3.5 h-3.5"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M6 18L18 6M6 6l12 12"
                        />
                    </svg>
                </button>
            </div>
        </div>

        <!-- Filters Bar -->
        <div
            v-if="pilots.length > 0"
            class="flex flex-col bg-eve-bg-1 border-b border-eve-border/50 text-[10px] shrink-0"
        >
            <!-- Row 1: Threat + Special filters -->
            <div
                class="flex items-center justify-between px-3 py-1 overflow-hidden"
            >
                <!-- Threat filters -->
                <div class="flex items-center gap-1 shrink-0">
                    <button
                        v-if="threatCounts.extreme"
                        @click="toggleThreatFilter('extreme')"
                        class="flex items-center gap-1 px-1.5 py-0.5 rounded transition-colors"
                        :class="
                            threatFilter === 'extreme'
                                ? 'bg-eve-threat-extreme/30'
                                : 'hover:bg-white/10'
                        "
                    >
                        <span
                            class="w-2 h-2 rounded-full bg-eve-threat-extreme"
                        ></span>
                        <span class="font-bold text-eve-threat-extreme">{{
                            threatCounts.extreme
                        }}</span>
                    </button>
                    <button
                        v-if="threatCounts.high"
                        @click="toggleThreatFilter('high')"
                        class="flex items-center gap-1 px-1.5 py-0.5 rounded transition-colors"
                        :class="
                            threatFilter === 'high'
                                ? 'bg-eve-threat-high/30'
                                : 'hover:bg-white/10'
                        "
                    >
                        <span
                            class="w-2 h-2 rounded-full bg-eve-threat-high"
                        ></span>
                        <span class="font-bold text-eve-threat-high">{{
                            threatCounts.high
                        }}</span>
                    </button>
                    <button
                        v-if="threatCounts.moderate"
                        @click="toggleThreatFilter('moderate')"
                        class="flex items-center gap-1 px-1.5 py-0.5 rounded transition-colors"
                        :class="
                            threatFilter === 'moderate'
                                ? 'bg-eve-threat-moderate/30'
                                : 'hover:bg-white/10'
                        "
                    >
                        <span
                            class="w-2 h-2 rounded-full bg-eve-threat-moderate"
                        ></span>
                        <span class="font-bold text-eve-threat-moderate">{{
                            threatCounts.moderate
                        }}</span>
                    </button>
                    <button
                        v-if="threatCounts.low"
                        @click="toggleThreatFilter('low')"
                        class="flex items-center gap-1 px-1.5 py-0.5 rounded transition-colors"
                        :class="
                            threatFilter === 'low'
                                ? 'bg-eve-threat-low/30'
                                : 'hover:bg-white/10'
                        "
                    >
                        <span
                            class="w-2 h-2 rounded-full bg-eve-threat-low"
                        ></span>
                        <span class="font-bold text-eve-threat-low">{{
                            threatCounts.low
                        }}</span>
                    </button>
                    <button
                        v-if="threatCounts.minimal"
                        @click="toggleThreatFilter('minimal')"
                        class="flex items-center gap-1 px-1.5 py-0.5 rounded transition-colors"
                        :class="
                            threatFilter === 'minimal'
                                ? 'bg-eve-threat-minimal/30'
                                : 'hover:bg-white/10'
                        "
                    >
                        <span
                            class="w-2 h-2 rounded-full bg-eve-threat-minimal"
                        ></span>
                        <span class="font-bold text-eve-threat-minimal">{{
                            threatCounts.minimal
                        }}</span>
                    </button>
                </div>

                <!-- Tag filters -->
                <div class="flex items-center gap-1 shrink-0 flex-wrap">
                    <button
                        v-if="tagCounts.super"
                        @click="toggleTag('super')"
                        class="px-1.5 py-0.5 font-bold rounded transition-colors"
                        :class="
                            selectedTags.has('super')
                                ? 'bg-rose-500/40 text-rose-200'
                                : 'bg-rose-500/20 text-rose-300 hover:bg-rose-500/30'
                        "
                    >
                        {{ tagCounts.super }} SUPER
                    </button>
                    <button
                        v-if="tagCounts.capital"
                        @click="toggleTag('capital')"
                        class="px-1.5 py-0.5 font-bold rounded transition-colors"
                        :class="
                            selectedTags.has('capital')
                                ? 'bg-amber-500/40 text-amber-200'
                                : 'bg-amber-500/20 text-amber-300 hover:bg-amber-500/30'
                        "
                    >
                        {{ tagCounts.capital }} CAPITAL
                    </button>
                    <button
                        v-if="tagCounts.blops"
                        @click="toggleTag('blops')"
                        class="px-1.5 py-0.5 font-bold rounded transition-colors"
                        :class="
                            selectedTags.has('blops')
                                ? 'bg-indigo-500/40 text-indigo-200'
                                : 'bg-indigo-500/20 text-indigo-300 hover:bg-indigo-500/30'
                        "
                    >
                        {{ tagCounts.blops }} BLACK OPS
                    </button>
                    <button
                        v-if="tagCounts.recon"
                        @click="toggleTag('recon')"
                        class="px-1.5 py-0.5 font-bold rounded transition-colors"
                        :class="
                            selectedTags.has('recon')
                                ? 'bg-teal-500/40 text-teal-200'
                                : 'bg-teal-500/20 text-teal-300 hover:bg-teal-500/30'
                        "
                    >
                        {{ tagCounts.recon }} RECON
                    </button>
                    <button
                        v-if="tagCounts.cyno"
                        @click="toggleTag('cyno')"
                        class="px-1.5 py-0.5 font-bold rounded transition-colors"
                        :class="
                            selectedTags.has('cyno')
                                ? 'bg-purple-500/40 text-purple-200'
                                : 'bg-purple-500/20 text-purple-300 hover:bg-purple-500/30'
                        "
                    >
                        {{ tagCounts.cyno }} CYNO
                    </button>
                </div>
            </div>
        </div>

        <!-- Column Headers -->
        <div
            v-if="pilots.length > 0"
            class="grid grid-cols-[32px_minmax(80px,1fr)_70px_45px_45px_60px_30px_48px_32px_32px_32px] gap-1 px-3 py-1 bg-eve-bg-2 border-b border-eve-border/50 text-[9px] font-semibold text-eve-text-3 uppercase shrink-0"
        >
            <button
                @click="handleSort('threat')"
                class="flex items-center justify-center hover:text-eve-text-1 transition-colors"
                :class="sortKey === 'threat' ? 'text-eve-cyan' : ''"
                title="Sort by threat"
            >
                <span v-if="sortKey === 'threat'" class="text-[8px]">{{
                    sortDirection === 'asc' ? '↑' : '↓'
                }}</span>
                <span v-else class="text-[8px]">⬤</span>
            </button>
            <button
                @click="handleSort('pilot')"
                class="text-left hover:text-eve-text-1 transition-colors"
                :class="sortKey === 'pilot' ? 'text-eve-cyan' : ''"
            >
                Pilot
                <span v-if="sortKey === 'pilot'" class="text-[8px]">{{
                    sortDirection === 'asc' ? '↑' : '↓'
                }}</span>
            </button>
            <button
                @click="handleSort('tags')"
                class="text-left hover:text-eve-text-1 transition-colors"
                :class="sortKey === 'tags' ? 'text-eve-cyan' : ''"
            >
                Tags
                <span v-if="sortKey === 'tags'" class="text-[8px]">{{
                    sortDirection === 'asc' ? '↑' : '↓'
                }}</span>
            </button>
            <button
                @click="handleSort('corp')"
                class="text-left hover:text-eve-text-1 transition-colors"
                :class="sortKey === 'corp' ? 'text-eve-cyan' : ''"
            >
                Corp
                <span v-if="sortKey === 'corp'" class="text-[8px]">{{
                    sortDirection === 'asc' ? '↑' : '↓'
                }}</span>
            </button>
            <button
                @click="handleSort('alliance')"
                class="text-left hover:text-eve-text-1 transition-colors"
                :class="sortKey === 'alliance' ? 'text-eve-cyan' : ''"
            >
                Ally
                <span v-if="sortKey === 'alliance'" class="text-[8px]">{{
                    sortDirection === 'asc' ? '↑' : '↓'
                }}</span>
            </button>
            <span>Ships</span>
            <button
                @click="handleSort('kd')"
                class="text-left hover:text-eve-text-1 transition-colors col-span-2"
                :class="sortKey === 'kd' ? 'text-eve-cyan' : ''"
            >
                K/D
                <span v-if="sortKey === 'kd'" class="text-[8px]">{{
                    sortDirection === 'asc' ? '↑' : '↓'
                }}</span>
            </button>
            <button
                @click="handleSort('ppk')"
                class="text-right hover:text-eve-text-1 transition-colors"
                :class="sortKey === 'ppk' ? 'text-eve-cyan' : ''"
            >
                PPK
                <span v-if="sortKey === 'ppk'" class="text-[8px]">{{
                    sortDirection === 'asc' ? '↑' : '↓'
                }}</span>
            </button>
            <button
                @click="handleSort('cpk')"
                class="text-right hover:text-eve-text-1 transition-colors"
                :class="sortKey === 'cpk' ? 'text-eve-cyan' : ''"
            >
                CPK
                <span v-if="sortKey === 'cpk'" class="text-[8px]">{{
                    sortDirection === 'asc' ? '↑' : '↓'
                }}</span>
            </button>
            <button
                @click="handleSort('active')"
                class="text-right hover:text-eve-text-1 transition-colors"
                :class="sortKey === 'active' ? 'text-eve-cyan' : ''"
            >
                Act
                <span v-if="sortKey === 'active'" class="text-[8px]">{{
                    sortDirection === 'asc' ? '↑' : '↓'
                }}</span>
            </button>
        </div>

        <!-- Pilots List -->
        <div
            v-if="pilots.length > 0"
            class="pilots-list flex-1 overflow-y-scroll overflow-x-hidden bg-eve-bg-0 relative"
        >
            <TransitionGroup name="row">
                <div
                    v-for="(pilot, index) in sortedPilots"
                    :key="pilot.character.id"
                    :style="{ '--i': index }"
                    class="grid grid-cols-[32px_minmax(80px,1fr)_70px_45px_45px_60px_30px_48px_32px_32px_32px] gap-1 items-center px-3 py-1 border-b border-eve-border/20 hover:bg-eve-bg-hover/50 transition-colors cursor-pointer"
                    @click="openZkill(pilot.character.id)"
                >
                    <!-- Threat dot -->
                    <span
                        class="w-2 h-2 rounded-full justify-self-center"
                        :class="{
                            'bg-eve-threat-extreme':
                                pilot.threat_level.toLowerCase() === 'extreme',
                            'bg-eve-threat-high':
                                pilot.threat_level.toLowerCase() === 'high',
                            'bg-eve-threat-moderate':
                                pilot.threat_level.toLowerCase() === 'moderate',
                            'bg-eve-threat-low':
                                pilot.threat_level.toLowerCase() === 'low',
                            'bg-eve-threat-minimal':
                                pilot.threat_level.toLowerCase() === 'minimal',
                            'bg-eve-threat-unknown':
                                pilot.threat_level.toLowerCase() === 'unknown',
                        }"
                    ></span>

                    <!-- Portrait + Pilot Name -->
                    <div class="flex items-center gap-1.5 min-w-0">
                        <img
                            v-if="pilot.character.id"
                            :src="getPortraitUrl(pilot.character.id)"
                            class="w-6 h-6 rounded shrink-0"
                        />
                        <div
                            v-else
                            class="w-6 h-6 rounded bg-eve-bg-3 shrink-0"
                        ></div>
                        <span
                            class="text-[11px] font-medium text-eve-text-1 truncate"
                            >{{ pilot.character.name }}</span
                        >
                    </div>

                    <!-- Tags -->
                    <div class="flex flex-wrap gap-0.5 overflow-hidden">
                        <span
                            v-for="flag in getFlagLabels(pilot.flags)"
                            :key="flag"
                            class="text-[7px] font-bold shrink-0"
                            :class="{
                                'text-purple-400': flag === 'CYNO',
                                'text-teal-400': flag === 'RECON',
                                'text-indigo-400': flag === 'BLACK OPS',
                                'text-amber-400': flag === 'CAPITAL',
                                'text-rose-400': flag === 'SUPER',
                                'text-sky-400': flag === 'SOLO',
                            }"
                            >{{ flag }}</span
                        >
                    </div>

                    <!-- Corp -->
                    <div
                        class="text-[10px] text-eve-text-3 truncate"
                        :title="pilot.character.corporation_name || ''"
                    >
                        {{ pilot.character.corporation_ticker || '—' }}
                    </div>

                    <!-- Alliance -->
                    <div
                        class="text-[10px] text-eve-text-3 truncate"
                        :title="pilot.character.alliance_name || ''"
                    >
                        {{ pilot.character.alliance_ticker || '—' }}
                    </div>

                    <!-- Ships -->
                    <div class="flex items-center gap-0.5 overflow-hidden">
                        <template v-if="pilot.zkill?.top_ships.length">
                            <img
                                v-for="ship in pilot.zkill.top_ships.slice(
                                    0,
                                    3
                                )"
                                :key="ship.ship_type_id"
                                :src="getShipIconUrl(ship.ship_type_id, 32)"
                                :title="ship.ship_name"
                                class="w-[18px] h-[18px] rounded bg-eve-bg-3 hover:ring-1 hover:ring-eve-cyan cursor-pointer"
                                @click.stop="
                                    openShipZkill(
                                        pilot.character.id,
                                        ship.ship_type_id
                                    )
                                "
                            />
                            <span
                                v-if="pilot.zkill.top_ships.length > 3"
                                class="text-[9px] text-eve-text-3"
                                >+{{ pilot.zkill.top_ships.length - 3 }}</span
                            >
                        </template>
                        <span v-else class="text-[10px] text-eve-text-3"
                            >—</span
                        >
                    </div>

                    <!-- K/D Ratio -->
                    <div class="text-[10px] font-mono tabular-nums text-right">
                        <span v-if="pilot.zkill" class="text-eve-text-2">
                            {{
                                getKdRatio(
                                    pilot.zkill.ships_destroyed,
                                    pilot.zkill.ships_lost
                                )
                            }}
                        </span>
                        <span v-else class="text-eve-text-3">—</span>
                    </div>

                    <!-- K/D Numbers -->
                    <div
                        class="text-[9px] font-mono tabular-nums leading-tight"
                    >
                        <template v-if="pilot.zkill">
                            <div class="text-eve-green">
                                +{{
                                    pilot.zkill.ships_destroyed.toLocaleString()
                                }}
                            </div>
                            <div class="text-eve-red">
                                -{{ pilot.zkill.ships_lost.toLocaleString() }}
                            </div>
                        </template>
                        <span v-else class="text-eve-text-3">—</span>
                    </div>

                    <!-- PPK -->
                    <div
                        class="text-[10px] font-mono tabular-nums text-right text-eve-text-2"
                    >
                        <span
                            v-if="
                                pilot.zkill && pilot.zkill.ships_destroyed > 0
                            "
                        >
                            {{
                                (
                                    pilot.zkill.points_destroyed /
                                    pilot.zkill.ships_destroyed
                                ).toFixed(0)
                            }}
                        </span>
                        <span v-else class="text-eve-text-3">—</span>
                    </div>

                    <!-- CPK -->
                    <div
                        class="text-[10px] font-mono tabular-nums text-right text-eve-text-2"
                    >
                        <span v-if="pilot.zkill && pilot.zkill.avg_attackers">
                            {{ pilot.zkill.avg_attackers.toFixed(1) }}
                        </span>
                        <span v-else class="text-eve-text-3">—</span>
                    </div>

                    <!-- Active -->
                    <div class="text-right text-[11px] font-mono">
                        <span
                            v-if="pilot.zkill"
                            :class="
                                pilot.zkill.active_pvp_kills > 20
                                    ? 'text-eve-orange font-bold'
                                    : 'text-eve-text-2'
                            "
                            >{{ pilot.zkill.active_pvp_kills }}</span
                        >
                        <span v-else class="text-eve-text-3">—</span>
                    </div>
                </div>
            </TransitionGroup>

            <!-- Empty state -->
            <div
                v-if="sortedPilots.length === 0"
                class="flex items-center justify-center py-6 text-eve-text-3"
            >
                <span class="text-xs">{{
                    pilots.length === 0
                        ? 'Scan pilots in main window'
                        : 'No matches'
                }}</span>
            </div>
        </div>
    </div>
</template>

<style>
.row-move,
.row-enter-active,
.row-leave-active {
    transition: all 0.3s ease;
}

.row-enter-active {
    transition-delay: calc(min(var(--i, 0) * 25ms, 400ms));
}

.row-enter-from {
    opacity: 0;
    transform: translateX(-20px);
}

.row-leave-to {
    opacity: 0;
}

.row-leave-active {
    position: absolute;
    left: 0;
    right: 0;
}

[data-tauri-drag-region] {
    -webkit-user-select: none;
    user-select: none;
    -webkit-app-region: drag;
    app-region: drag;
}

[data-tauri-drag-region] button {
    -webkit-app-region: no-drag;
    app-region: no-drag;
}
</style>
