<script setup lang="ts">
import { ref, computed } from "vue";
import type { PilotIntel } from "../types";
import PilotRow from "./PilotRow.vue";
import SortableHeader from "./SortableHeader.vue";
import { useSettings } from "../composables/useSettings";

const props = defineProps<{
  pilots: PilotIntel[];
}>();

const { settings } = useSettings();

const expandedPilot = ref<number | null>(null);
const sortKey = computed({
  get: () => settings.value.sortColumn,
  set: (val) => { settings.value.sortColumn = val; }
});
const sortDirection = computed({
  get: () => settings.value.sortDirection,
  set: (val) => { settings.value.sortDirection = val; }
});

const THREAT_ORDER: Record<string, number> = {
  extreme: 5,
  high: 4,
  moderate: 3,
  low: 2,
  minimal: 1,
  unknown: 0,
};

function toggleExpand(id: number) {
  expandedPilot.value = expandedPilot.value === id ? null : id;
}

function handleSort(key: string) {
  if (sortKey.value === key) {
    sortDirection.value = sortDirection.value === "asc" ? "desc" : "asc";
  } else {
    sortKey.value = key;
    sortDirection.value = "desc";
  }
}

const sortedPilots = computed(() => {
  const sorted = [...props.pilots].sort((a, b) => {
    let comparison = 0;
    
    switch (sortKey.value) {
      case "threat":
        comparison = (THREAT_ORDER[a.threat_level.toLowerCase()] || 0) - 
                     (THREAT_ORDER[b.threat_level.toLowerCase()] || 0);
        break;
      case "pilot":
        comparison = a.character.name.localeCompare(b.character.name);
        break;
      case "corporation":
        comparison = (a.character.corporation_name || "").localeCompare(b.character.corporation_name || "");
        break;
      case "alliance":
        comparison = (a.character.alliance_name || "").localeCompare(b.character.alliance_name || "");
        break;
      case "kills":
        comparison = (a.zkill?.ships_destroyed || 0) - (b.zkill?.ships_destroyed || 0);
        break;
      case "deaths":
        comparison = (a.zkill?.ships_lost || 0) - (b.zkill?.ships_lost || 0);
        break;
      case "isk":
        comparison = (a.zkill?.isk_destroyed || 0) - (b.zkill?.isk_destroyed || 0);
        break;
      case "active":
        comparison = (a.zkill?.active_pvp_kills || 0) - (b.zkill?.active_pvp_kills || 0);
        break;
      case "danger":
        comparison = (a.zkill?.danger_ratio || 0) - (b.zkill?.danger_ratio || 0);
        break;
      case "ppk": {
        const ppkA = a.zkill ? (a.zkill.ships_destroyed > 0 ? a.zkill.points_destroyed / a.zkill.ships_destroyed : 0) : 0;
        const ppkB = b.zkill ? (b.zkill.ships_destroyed > 0 ? b.zkill.points_destroyed / b.zkill.ships_destroyed : 0) : 0;
        comparison = ppkA - ppkB;
        break;
      }
    }
    
    return sortDirection.value === "asc" ? comparison : -comparison;
  });
  
  return sorted;
});
</script>

<template>
  <main class="flex-1 flex flex-col overflow-hidden">
    <div class="grid grid-cols-[54px_minmax(120px,1fr)_minmax(100px,1.2fr)_minmax(80px,1fr)_160px_100px_90px_50px_50px] gap-2 px-4 py-2 bg-eve-bg-2 border-b border-eve-border text-[10px] font-semibold tracking-wider text-eve-text-3 uppercase shrink-0">
      <SortableHeader 
        label="Threat" 
        sort-key="threat" 
        :current-sort="sortKey" 
        :sort-direction="sortDirection"
        @sort="handleSort"
      />
      <SortableHeader 
        sort-key="pilot" 
        :current-sort="sortKey" 
        :sort-direction="sortDirection"
        @sort="handleSort"
      >
        <span class="flex items-center gap-2">
          Pilot
          <span class="font-mono text-[10px] bg-eve-bg-3 px-1.5 py-0.5 rounded text-eve-cyan">{{ pilots.length }}</span>
        </span>
      </SortableHeader>
      <SortableHeader 
        label="Corporation" 
        sort-key="corporation" 
        :current-sort="sortKey" 
        :sort-direction="sortDirection"
        @sort="handleSort"
      />
      <SortableHeader 
        label="Alliance" 
        sort-key="alliance" 
        :current-sort="sortKey" 
        :sort-direction="sortDirection"
        @sort="handleSort"
      />
      <span>Ships</span>
      <SortableHeader 
        label="K / D" 
        sort-key="kills" 
        :current-sort="sortKey" 
        :sort-direction="sortDirection"
        @sort="handleSort"
      />
      <SortableHeader 
        label="ISK" 
        sort-key="isk" 
        :current-sort="sortKey" 
        :sort-direction="sortDirection"
        @sort="handleSort"
      />
      <SortableHeader 
        label="PPK" 
        sort-key="ppk" 
        :current-sort="sortKey" 
        :sort-direction="sortDirection"
        align="right"
        @sort="handleSort"
      />
      <SortableHeader 
        label="Active" 
        sort-key="active" 
        :current-sort="sortKey" 
        :sort-direction="sortDirection"
        align="right"
        @sort="handleSort"
      />
    </div>

    <div class="flex-1 overflow-y-scroll relative">
      <TransitionGroup name="row">
        <div
          v-for="(pilot, index) in sortedPilots"
          :key="pilot.character.id || pilot.character.name"
          :style="{ '--i': index }"
        >
          <PilotRow
            :pilot="pilot"
            :expanded="expandedPilot === pilot.character.id"
            @toggle="toggleExpand(pilot.character.id)"
          />
        </div>
      </TransitionGroup>
    </div>
  </main>
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
</style>
