<script setup lang="ts">
import type { PilotIntel } from '../types'
import {
    formatIsk,
    formatPpk,
    getPortraitUrl,
    getShipIconUrl,
} from '../utils/format'
import { getFlagLabels } from '../utils/intel'
import ThreatBadge from './ThreatBadge.vue'
import PilotDetails from './PilotDetails.vue'

defineProps<{
    pilot: PilotIntel
    expanded: boolean
}>()

const emit = defineEmits<{
    toggle: []
}>()
</script>

<template>
    <div
        class="border-b border-eve-border bg-eve-bg-1 transition-colors hover:bg-eve-bg-hover border-l-[3px] data-[threat=extreme]:border-l-eve-threat-extreme data-[threat=high]:border-l-eve-threat-high data-[threat=moderate]:border-l-eve-threat-moderate data-[threat=low]:border-l-eve-threat-low data-[threat=minimal]:border-l-eve-threat-minimal data-[threat=unknown]:border-l-eve-threat-unknown"
        :data-threat="pilot.threat_level.toLowerCase()"
    >
        <div
            class="grid grid-cols-[54px_minmax(120px,1fr)_120px_minmax(100px,1.2fr)_minmax(80px,1fr)_140px_35px_65px_90px_50px_50px_50px] gap-2 items-center px-4 py-1.5 cursor-pointer"
            @click="emit('toggle')"
        >
            <!-- Threat Badge -->
            <div>
                <ThreatBadge :level="pilot.threat_level" compact />
            </div>

            <!-- Pilot -->
            <div class="flex items-center gap-2 min-w-0">
                <img
                    v-if="pilot.character.id"
                    :src="getPortraitUrl(pilot.character.id)"
                    class="w-[26px] h-[26px] rounded shrink-0"
                />
                <div
                    v-else
                    class="w-[26px] h-[26px] rounded bg-eve-bg-3 flex items-center justify-center text-eve-text-3 text-xs shrink-0"
                >
                    ?
                </div>
                <span class="font-semibold text-sm truncate">{{
                    pilot.character.name
                }}</span>
            </div>

            <!-- Tags -->
            <div class="flex items-center gap-1 flex-wrap">
                <span
                    v-for="flag in getFlagLabels(pilot.flags)"
                    :key="flag"
                    class="text-[9px] font-bold px-1.5 py-0.5 rounded"
                    :class="{
                        'bg-purple-500/20 text-purple-400': flag === 'CYNO',
                        'bg-teal-500/20 text-teal-400': flag === 'RECON',
                        'bg-indigo-500/20 text-indigo-400':
                            flag === 'BLACK OPS',
                        'bg-amber-500/20 text-amber-400': flag === 'CAPITAL',
                        'bg-rose-500/20 text-rose-400': flag === 'SUPER',
                        'bg-sky-500/20 text-sky-400': flag === 'SOLO',
                    }"
                    >{{ flag }}</span
                >
            </div>

            <!-- Corporation -->
            <div
                class="flex items-center gap-1.5 text-xs text-eve-text-2 min-w-0"
                :title="pilot.character.corporation_name || ''"
            >
                <span
                    v-if="pilot.character.corporation_ticker"
                    class="font-mono text-[10px] text-eve-text-3 shrink-0"
                    >[{{ pilot.character.corporation_ticker }}]</span
                >
                <span class="truncate">{{
                    pilot.character.corporation_name || '—'
                }}</span>
            </div>

            <!-- Alliance -->
            <div
                class="flex items-center gap-1.5 text-xs text-eve-text-2 min-w-0"
                :title="pilot.character.alliance_name || ''"
            >
                <span
                    v-if="pilot.character.alliance_ticker"
                    class="font-mono text-[10px] text-eve-text-3 shrink-0"
                    >[{{ pilot.character.alliance_ticker }}]</span
                >
                <span class="truncate">{{
                    pilot.character.alliance_name || '—'
                }}</span>
            </div>

            <!-- Ships -->
            <div class="flex items-center gap-1">
                <template v-if="pilot.zkill?.top_ships.length">
                    <div
                        v-for="ship in pilot.zkill.top_ships.slice(0, 5)"
                        :key="ship.ship_type_id"
                        class="w-[26px] h-[26px] rounded overflow-hidden bg-eve-bg-3"
                        :title="`${ship.ship_name} (${ship.kills})`"
                    >
                        <img
                            :src="getShipIconUrl(ship.ship_type_id, 64)"
                            :alt="ship.ship_name"
                            class="w-full h-full object-cover"
                        />
                    </div>
                    <span
                        v-if="pilot.zkill.top_ships.length > 5"
                        class="text-[10px] text-eve-text-3 ml-0.5"
                    >
                        +{{ pilot.zkill.top_ships.length - 5 }}
                    </span>
                </template>
                <span v-else class="text-eve-text-3">—</span>
            </div>

            <!-- K/D Ratio -->
            <div
                class="font-mono text-sm text-eve-text-2 tabular-nums text-right"
            >
                <span v-if="pilot.zkill">{{
                    pilot.zkill.ships_lost > 0
                        ? (
                              pilot.zkill.ships_destroyed /
                              pilot.zkill.ships_lost
                          ).toFixed(1)
                        : pilot.zkill.ships_destroyed > 0
                          ? '∞'
                          : '0'
                }}</span>
                <span v-else class="text-eve-text-3">—</span>
            </div>

            <!-- K/D Numbers -->
            <div
                class="font-mono text-[11px] flex flex-col leading-tight tabular-nums"
            >
                <template v-if="pilot.zkill">
                    <span class="text-eve-green"
                        >+{{
                            pilot.zkill.ships_destroyed.toLocaleString()
                        }}</span
                    >
                    <span class="text-eve-red"
                        >-{{ pilot.zkill.ships_lost.toLocaleString() }}</span
                    >
                </template>
                <span v-else class="text-eve-text-3">—</span>
            </div>

            <!-- ISK -->
            <div class="font-mono text-[11px] flex flex-col leading-tight pl-2">
                <template v-if="pilot.zkill">
                    <span class="text-eve-green"
                        >+{{ formatIsk(pilot.zkill.isk_destroyed) }}</span
                    >
                    <span class="text-eve-red"
                        >-{{ formatIsk(pilot.zkill.isk_lost) }}</span
                    >
                </template>
                <span v-else class="text-eve-text-3">—</span>
            </div>

            <!-- PPK -->
            <div class="text-right font-mono text-xs">
                <span v-if="pilot.zkill" class="text-eve-text-2">
                    {{
                        formatPpk(
                            pilot.zkill.points_destroyed,
                            pilot.zkill.ships_destroyed
                        )
                    }}
                </span>
                <span v-else class="text-eve-text-3">—</span>
            </div>

            <!-- CPK -->
            <div class="text-right font-mono text-xs">
                <span v-if="pilot.zkill" class="text-eve-text-2">
                    {{ pilot.zkill.avg_attackers.toFixed(1) }}
                </span>
                <span v-else class="text-eve-text-3">—</span>
            </div>

            <!-- Active -->
            <div class="text-right font-mono">
                <span
                    v-if="pilot.zkill"
                    class="text-sm data-[hot=true]:text-eve-orange data-[hot=true]:font-semibold data-[hot=false]:text-eve-text-2"
                    :data-hot="pilot.zkill.active_pvp_kills > 20"
                >
                    {{ pilot.zkill.active_pvp_kills }}
                </span>
                <span v-else class="text-eve-text-3">—</span>
            </div>
        </div>

        <PilotDetails
            v-if="expanded && pilot.zkill"
            :pilot="pilot"
            :zkill="pilot.zkill"
        />

        <div
            v-if="pilot.error"
            class="px-4 py-1 pl-14 text-xs text-eve-red bg-eve-red/5"
        >
            {{ pilot.error }}
        </div>
    </div>
</template>
