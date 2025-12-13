<script setup lang="ts">
import { computed } from "vue";
import { X } from "lucide-vue-next";
import type { PilotIntel } from "../types";
import FilterGroup from "./FilterGroup.vue";

const props = defineProps<{
  pilots: PilotIntel[];
  selectedCorps: Set<string>;
  selectedAlliances: Set<string>;
}>();

const emit = defineEmits<{
  toggleCorp: [name: string];
  toggleAlliance: [name: string];
  clearFilters: [];
}>();

interface GroupInfo {
  id: number;
  name: string;
  ticker: string;
  count: number;
}

const corporations = computed<GroupInfo[]>(() => {
  const groups = new Map<string, { id: number; ticker: string; count: number }>();
  for (const p of props.pilots) {
    const name = p.character.corporation_name || "Unknown";
    const id = p.character.corporation_id || 0;
    const ticker = p.character.corporation_ticker || "";
    const existing = groups.get(name);
    if (existing) {
      existing.count++;
    } else {
      groups.set(name, { id, ticker, count: 1 });
    }
  }
  return Array.from(groups.entries())
    .map(([name, { id, ticker, count }]) => ({ id, name, ticker, count }))
    .sort((a, b) => b.count - a.count);
});

const alliances = computed<GroupInfo[]>(() => {
  const groups = new Map<string, { id: number; ticker: string; count: number }>();
  for (const p of props.pilots) {
    const name = p.character.alliance_name;
    const id = p.character.alliance_id;
    const ticker = p.character.alliance_ticker || "";
    if (name && id) {
      const existing = groups.get(name);
      if (existing) {
        existing.count++;
      } else {
        groups.set(name, { id, ticker, count: 1 });
      }
    }
  }
  return Array.from(groups.entries())
    .map(([name, { id, ticker, count }]) => ({ id, name, ticker, count }))
    .sort((a, b) => b.count - a.count);
});

const hasFilters = computed(() => {
  return props.selectedCorps.size > 0 || props.selectedAlliances.size > 0;
});

function getCorpLogo(id: number): string {
  return `https://images.evetech.net/corporations/${id}/logo?size=32`;
}

function getAllianceLogo(id: number): string {
  return `https://images.evetech.net/alliances/${id}/logo?size=32`;
}
</script>

<template>
  <aside class="w-64 bg-eve-bg-1 border-l border-eve-border flex flex-col overflow-hidden">
    <div class="flex justify-between items-center px-4 py-3 border-b border-eve-border shrink-0">
      <span class="text-[10px] font-semibold tracking-[0.15em] text-eve-text-3">FILTERS</span>
      <button
        class="w-5 h-5 flex items-center justify-center rounded text-eve-text-3 cursor-pointer transition-all hover:bg-eve-bg-hover hover:text-eve-text-1"
        :class="hasFilters ? 'opacity-100' : 'opacity-0 pointer-events-none'"
        @click="emit('clearFilters')"
        title="Clear filters"
      >
        <X class="w-3 h-3" />
      </button>
    </div>

    <!-- Alliances -->
    <div v-if="alliances.length > 0" class="p-3 border-b border-eve-border">
      <FilterGroup
        title="Alliances"
        :items="alliances"
        :selected="selectedAlliances"
        :get-logo-url="getAllianceLogo"
        @toggle="emit('toggleAlliance', $event)"
      />
    </div>

    <!-- Corporations -->
    <div class="p-3 flex-1 overflow-hidden">
      <FilterGroup
        title="Corporations"
        :items="corporations"
        :selected="selectedCorps"
        :get-logo-url="getCorpLogo"
        @toggle="emit('toggleCorp', $event)"
      />
    </div>
  </aside>
</template>
