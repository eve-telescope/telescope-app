<script setup lang="ts">
import { ref, computed } from 'vue'
import { StickyNote } from 'lucide-vue-next'
import { Badge } from '@/components/ui/badge'
import {
    Popover,
    PopoverContent,
    PopoverTrigger,
} from '@/components/ui/popover'
import type { PilotIntel } from '../types'
import {
    formatIsk,
    formatPpk,
    getPortraitUrl,
    getShipIconUrl,
} from '../utils/format'
import { resolvePilotAnnotations } from '../stores/intel'
import { formatAnnotationScope } from '../utils/annotations'
import { getPilotTags } from '../utils/pilotTags'
import ThreatBadge from './ThreatBadge.vue'
import PilotDetails from './PilotDetails.vue'
import IntelContextMenu from './IntelContextMenu.vue'

const props = defineProps<{
    pilot: PilotIntel
    expanded: boolean
}>()

const pilotIntel = computed(() => resolvePilotAnnotations(props.pilot))

const allTags = computed(() => getPilotTags(props.pilot))

const pilotNotes = computed(() =>
    pilotIntel.value
        .filter((match) => match.annotation.note)
        .map((match) => ({
            key: match.key,
            scope: match.scope,
            tags: match.annotation.tags,
            color: match.annotation.color,
            note: match.annotation.note!,
            networkName: match.annotation.networkName,
            targetName: match.annotation.targetName,
        }))
)

const showNotes = ref(false)

const emit = defineEmits<{
    toggle: []
}>()
</script>

<template>
    <tbody>
        <IntelContextMenu
            :character-id="pilot.character.id"
            :character-name="pilot.character.name"
            :corporation-id="pilot.character.corporation_id"
            :corporation-name="pilot.character.corporation_name"
            :alliance-id="pilot.character.alliance_id"
            :alliance-name="pilot.character.alliance_name"
        >
            <tr
                class="border-b border-eve-border bg-eve-bg-1 transition-colors hover:bg-eve-bg-hover border-l-[3px] cursor-pointer data-[threat=extreme]:border-l-eve-threat-extreme data-[threat=high]:border-l-eve-threat-high data-[threat=moderate]:border-l-eve-threat-moderate data-[threat=low]:border-l-eve-threat-low data-[threat=minimal]:border-l-eve-threat-minimal data-[threat=unknown]:border-l-eve-threat-unknown"
                :data-threat="pilot.threat_level.toLowerCase()"
                @click="emit('toggle')"
            >
                <!-- Threat -->
                <td class="px-2 py-1.5">
                    <ThreatBadge :level="pilot.threat_level" compact />
                </td>

                <!-- Pilot -->
                <td class="px-2 py-1.5">
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
                </td>

                <!-- Tags -->
                <td class="px-2 py-1.5">
                    <div class="flex items-center gap-1 flex-wrap">
                        <Badge
                            v-for="t in allTags"
                            :key="t.key"
                            variant="secondary"
                            :style="{
                                backgroundColor: (t.color || '#94A3B8') + '22',
                                color: t.color || '#CBD5E1',
                            }"
                            >{{ t.tag }}</Badge
                        >
                        <Popover
                            v-if="pilotNotes.length > 0"
                            v-model:open="showNotes"
                        >
                            <PopoverTrigger as-child>
                                <button
                                    class="p-0.5 rounded text-eve-orange hover:bg-eve-orange/10 transition-colors"
                                    :title="`${pilotNotes.length} note(s)`"
                                    @click.stop
                                >
                                    <StickyNote
                                        class="w-3 h-3"
                                        :stroke-width="2"
                                    />
                                </button>
                            </PopoverTrigger>
                            <PopoverContent class="space-y-2" @click.stop>
                                <div v-for="note in pilotNotes" :key="note.key">
                                    <div class="flex items-center gap-1.5 mb-1">
                                        <Badge
                                            variant="secondary"
                                            :style="{
                                                backgroundColor:
                                                    (note.color || '#94A3B8') +
                                                    '33',
                                                color: note.color || '#CBD5E1',
                                            }"
                                            >{{
                                                formatAnnotationScope(
                                                    note.scope
                                                )
                                            }}</Badge
                                        >
                                        <span
                                            class="text-[9px] text-eve-text-3"
                                            >{{ note.networkName }}</span
                                        >
                                        <span
                                            class="text-[9px] text-eve-text-3 truncate"
                                            >{{ note.targetName }}</span
                                        >
                                    </div>
                                    <div class="flex flex-wrap gap-1 mb-1">
                                        <Badge
                                            v-for="tag in note.tags"
                                            :key="`${note.key}:${tag}`"
                                            variant="secondary"
                                            :style="{
                                                backgroundColor:
                                                    (note.color || '#94A3B8') +
                                                    '33',
                                                color: note.color || '#CBD5E1',
                                            }"
                                            >{{ tag }}</Badge
                                        >
                                    </div>
                                    <div
                                        class="text-[11px] text-eve-text-2 leading-relaxed whitespace-pre-wrap"
                                    >
                                        {{ note.note }}
                                    </div>
                                </div>
                            </PopoverContent>
                        </Popover>
                    </div>
                </td>

                <!-- Corporation -->
                <td class="px-2 py-1.5">
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
                </td>

                <!-- Alliance -->
                <td class="px-2 py-1.5">
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
                </td>

                <!-- Ships -->
                <td class="px-2 py-1.5">
                    <div class="flex items-center gap-1">
                        <template v-if="pilot.zkill?.top_ships.length">
                            <div
                                v-for="ship in pilot.zkill.top_ships.slice(
                                    0,
                                    5
                                )"
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
                                >+{{ pilot.zkill.top_ships.length - 5 }}</span
                            >
                        </template>
                        <span v-else class="text-eve-text-3">—</span>
                    </div>
                </td>

                <!-- K/D Ratio -->
                <td
                    class="px-2 py-1.5 font-mono text-sm text-eve-text-2 tabular-nums text-right"
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
                </td>

                <!-- K/D Numbers -->
                <td class="px-2 py-1.5 font-mono text-[11px] tabular-nums">
                    <div class="flex flex-col leading-tight">
                        <template v-if="pilot.zkill">
                            <span class="text-eve-green"
                                >+{{
                                    pilot.zkill.ships_destroyed.toLocaleString()
                                }}</span
                            >
                            <span class="text-eve-red"
                                >-{{
                                    pilot.zkill.ships_lost.toLocaleString()
                                }}</span
                            >
                        </template>
                        <span v-else class="text-eve-text-3">—</span>
                    </div>
                </td>

                <!-- ISK -->
                <td class="px-2 py-1.5 font-mono text-[11px]">
                    <div class="flex flex-col leading-tight">
                        <template v-if="pilot.zkill">
                            <span class="text-eve-green"
                                >+{{
                                    formatIsk(pilot.zkill.isk_destroyed)
                                }}</span
                            >
                            <span class="text-eve-red"
                                >-{{ formatIsk(pilot.zkill.isk_lost) }}</span
                            >
                        </template>
                        <span v-else class="text-eve-text-3">—</span>
                    </div>
                </td>

                <!-- PPK -->
                <td class="px-2 py-1.5 text-right font-mono text-xs">
                    <span v-if="pilot.zkill" class="text-eve-text-2">{{
                        formatPpk(
                            pilot.zkill.points_destroyed,
                            pilot.zkill.ships_destroyed
                        )
                    }}</span>
                    <span v-else class="text-eve-text-3">—</span>
                </td>

                <!-- CPK -->
                <td class="px-2 py-1.5 text-right font-mono text-xs">
                    <span v-if="pilot.zkill" class="text-eve-text-2">{{
                        pilot.zkill.avg_attackers.toFixed(1)
                    }}</span>
                    <span v-else class="text-eve-text-3">—</span>
                </td>

                <!-- Active -->
                <td class="px-2 py-1.5 text-right font-mono">
                    <span
                        v-if="pilot.zkill"
                        class="text-sm data-[hot=true]:text-eve-orange data-[hot=true]:font-semibold data-[hot=false]:text-eve-text-2"
                        :data-hot="pilot.zkill.active_pvp_kills > 20"
                        >{{ pilot.zkill.active_pvp_kills }}</span
                    >
                    <span v-else class="text-eve-text-3">—</span>
                </td>
            </tr>
        </IntelContextMenu>

        <!-- Expanded details -->
        <tr v-if="expanded && pilot.zkill">
            <td :colspan="12" class="p-0">
                <PilotDetails :pilot="pilot" :zkill="pilot.zkill" />
            </td>
        </tr>

        <!-- Error -->
        <tr v-if="pilot.error">
            <td
                :colspan="12"
                class="px-4 py-1 pl-14 text-xs text-eve-red bg-eve-red/5"
            >
                {{ pilot.error }}
            </td>
        </tr>
    </tbody>
</template>
