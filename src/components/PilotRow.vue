<script setup lang="ts">
import { ref, computed, nextTick, type FunctionalComponent } from 'vue'
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
import {
    getPilotTags,
    DEFAULT_TAG_COLOR,
    DEFAULT_TAG_TEXT_COLOR,
} from '../utils/pilotTags'
import AffiliationBadge from './AffiliationBadge.vue'
import ThreatBadge from './ThreatBadge.vue'
import PilotDetails from './PilotDetails.vue'
import IntelContextMenu from './IntelContextMenu.vue'

const props = defineProps<{
    pilot: PilotIntel
    expanded: boolean
}>()

const pilotIntel = computed(() => resolvePilotAnnotations(props.pilot))

// Reuse the resolved annotations so resolution runs once per row, not
// twice. Styles are precomputed here so the template binds stable objects
// instead of rebuilding style objects/strings on every render.
const allTags = computed(() =>
    getPilotTags(props.pilot, pilotIntel.value).map((t) => ({
        key: t.key,
        tag: t.tag,
        style: {
            backgroundColor: (t.color || DEFAULT_TAG_COLOR) + '22',
            color: t.color || DEFAULT_TAG_TEXT_COLOR,
        },
    }))
)

const pilotNotes = computed(() =>
    pilotIntel.value
        .filter((match) => match.annotation.note)
        .map((match) => ({
            key: match.key,
            scope: match.scope,
            tags: match.annotation.tags,
            note: match.annotation.note!,
            networkName: match.annotation.networkName,
            targetName: match.annotation.targetName,
            badgeStyle: {
                backgroundColor:
                    (match.annotation.color || DEFAULT_TAG_COLOR) + '33',
                color: match.annotation.color || DEFAULT_TAG_TEXT_COLOR,
            },
        }))
)

const showNotes = ref(false)

const emit = defineEmits<{
    toggle: []
}>()

// The context menu (trigger wrapper, four computeds, form state) is only
// mounted on the first right-click instead of eagerly on every row —
// hundreds of rows would otherwise pay its setup cost during a scan.
const menuMounted = ref(false)
const rowEl = ref<HTMLElement | null>(null)

const BareSlot: FunctionalComponent = (_, { slots }) => slots.default?.()

const menuProps = computed(() =>
    menuMounted.value
        ? {
              characterId: props.pilot.character.id,
              characterName: props.pilot.character.name,
              corporationId: props.pilot.character.corporation_id,
              corporationName: props.pilot.character.corporation_name,
              allianceId: props.pilot.character.alliance_id,
              allianceName: props.pilot.character.alliance_name,
          }
        : undefined
)

async function mountMenu(e: MouseEvent) {
    // Once mounted, reka-ui's trigger on the row handles the event itself.
    if (menuMounted.value) return
    e.preventDefault()
    menuMounted.value = true
    await nextTick()
    // Replay the right-click on the freshly mounted trigger (the original
    // event fired before it existed) so the first click still opens the menu.
    rowEl.value?.dispatchEvent(
        new MouseEvent('contextmenu', {
            bubbles: true,
            cancelable: true,
            clientX: e.clientX,
            clientY: e.clientY,
        })
    )
}
</script>

<template>
    <!-- Div rows instead of table markup: WKWebView can't composite animated
         table elements, and the scan cross-fade animates these rows. -->
    <div
        class="border-b border-eve-border [content-visibility:auto] [contain-intrinsic-size:auto_39px]"
        @contextmenu="mountMenu"
    >
        <component
            :is="menuMounted ? IntelContextMenu : BareSlot"
            v-bind="menuProps"
        >
            <div
                ref="rowEl"
                class="pilot-grid bg-eve-bg-1 transition-colors hover:bg-eve-bg-hover border-l-[3px] cursor-pointer data-[threat=extreme]:border-l-eve-threat-extreme data-[threat=high]:border-l-eve-threat-high data-[threat=moderate]:border-l-eve-threat-moderate data-[threat=low]:border-l-eve-threat-low data-[threat=minimal]:border-l-eve-threat-minimal data-[threat=unknown]:border-l-eve-threat-unknown"
                :data-threat="pilot.threat_level.toLowerCase()"
                @click="emit('toggle')"
            >
                <!-- Threat -->
                <div class="px-2 py-1.5">
                    <ThreatBadge :level="pilot.threat_level" compact />
                </div>

                <!-- Pilot -->
                <div class="px-2 py-1.5 min-w-0">
                    <div class="flex items-center gap-2 min-w-0">
                        <img
                            v-if="pilot.character.id"
                            :src="getPortraitUrl(pilot.character.id)"
                            loading="lazy"
                            decoding="async"
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
                </div>

                <!-- Tags -->
                <div class="px-2 py-1.5">
                    <div class="flex items-center gap-1 flex-wrap">
                        <Badge
                            v-for="t in allTags"
                            :key="t.key"
                            variant="secondary"
                            :style="t.style"
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
                                            :style="note.badgeStyle"
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
                                            :style="note.badgeStyle"
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
                </div>

                <!-- Corporation -->
                <div class="px-2 py-1.5 min-w-0">
                    <AffiliationBadge
                        type="corporation"
                        :entity-id="pilot.character.corporation_id"
                        :name="pilot.character.corporation_name"
                    />
                </div>

                <!-- Alliance -->
                <div class="px-2 py-1.5 min-w-0">
                    <AffiliationBadge
                        type="alliance"
                        :entity-id="pilot.character.alliance_id"
                        :name="pilot.character.alliance_name"
                    />
                </div>

                <!-- Ships -->
                <div class="px-2 py-1.5">
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
                                    loading="lazy"
                                    decoding="async"
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
                </div>

                <!-- K/D Ratio -->
                <div
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
                </div>

                <!-- K/D Numbers -->
                <div class="px-2 py-1.5 font-mono text-[11px] tabular-nums">
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
                </div>

                <!-- ISK -->
                <div class="px-2 py-1.5 font-mono text-[11px]">
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
                </div>

                <!-- PPK -->
                <div class="px-2 py-1.5 text-right font-mono text-xs">
                    <span v-if="pilot.zkill" class="text-eve-text-2">{{
                        formatPpk(
                            pilot.zkill.points_destroyed,
                            pilot.zkill.ships_destroyed
                        )
                    }}</span>
                    <span v-else class="text-eve-text-3">—</span>
                </div>

                <!-- CPK -->
                <div class="px-2 py-1.5 text-right font-mono text-xs">
                    <span v-if="pilot.zkill" class="text-eve-text-2">{{
                        pilot.zkill.avg_attackers.toFixed(1)
                    }}</span>
                    <span v-else class="text-eve-text-3">—</span>
                </div>

                <!-- Active -->
                <div class="px-2 py-1.5 text-right font-mono">
                    <span
                        v-if="pilot.zkill"
                        class="text-sm data-[hot=true]:text-eve-orange data-[hot=true]:font-semibold data-[hot=false]:text-eve-text-2"
                        :data-hot="pilot.zkill.active_pvp_kills > 20"
                        >{{ pilot.zkill.active_pvp_kills }}</span
                    >
                    <span v-else class="text-eve-text-3">—</span>
                </div>
            </div>
        </component>

        <!-- Expanded details -->
        <PilotDetails
            v-if="expanded && pilot.zkill"
            :pilot="pilot"
            :zkill="pilot.zkill"
        />

        <!-- Error -->
        <div
            v-if="pilot.error"
            class="px-4 py-1 pl-14 text-xs text-eve-red bg-eve-red/5"
        >
            {{ pilot.error }}
        </div>
    </div>
</template>
