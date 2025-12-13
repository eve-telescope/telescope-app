<script setup lang="ts">
import { usePilots } from "./composables/usePilots";
import { useGlobalShortcut } from "./composables/useGlobalShortcut";
import { useDeepLink } from "./composables/useDeepLink";
import TitleBar from "./components/TitleBar.vue";
import InputPanel from "./components/InputPanel.vue";
import PilotTable from "./components/PilotTable.vue";
import FilterSidebar from "./components/FilterSidebar.vue";
import EmptyState from "./components/EmptyState.vue";

const {
  pilotNames,
  pilots,
  filteredPilots,
  loading,
  pilotCount,
  selectedCorps,
  selectedAlliances,
  lookupPilots,
  toggleCorp,
  toggleAlliance,
  clearFilters,
  clear,
} = usePilots();

function handleGlobalPaste(text: string) {
  lookupPilots(text);
}

function handleDeepLinkPilots(pilots: string) {
  lookupPilots(pilots);
}

const { 
  displayShortcut, 
  isActive: shortcutActive, 
  error: shortcutError,
  updateShortcut 
} = useGlobalShortcut(handleGlobalPaste);

useDeepLink(handleDeepLinkPilots);
</script>

<template>
  <div class="h-screen flex flex-col overflow-hidden bg-eve-bg-0">
    <TitleBar :pilot-count="pilots.length" />

    <div class="flex-1 flex overflow-hidden">
      <!-- Left: Input Panel -->
      <InputPanel
        v-model:pilot-names="pilotNames"
        :loading="loading"
        :pilot-count="pilotCount"
        :pilots="pilots"
        :shortcut="displayShortcut()"
        :shortcut-active="shortcutActive"
        :shortcut-error="shortcutError"
        @scan="lookupPilots"
        @clear="clear"
        @update:shortcut="updateShortcut"
      />

      <!-- Center: Results -->
      <div class="flex-1 flex flex-col overflow-hidden">
        <PilotTable v-if="pilots.length > 0" :pilots="filteredPilots" />
        <EmptyState v-else-if="!loading" />
        <div v-else class="flex-1 flex items-center justify-center">
          <div class="w-6 h-6 border-2 border-eve-cyan border-t-transparent rounded-full animate-spin"></div>
        </div>
      </div>

      <!-- Right: Filters -->
      <FilterSidebar
        v-if="pilots.length > 0"
        :pilots="pilots"
        :selected-corps="selectedCorps"
        :selected-alliances="selectedAlliances"
        @toggle-corp="toggleCorp"
        @toggle-alliance="toggleAlliance"
        @clear-filters="clearFilters"
      />
    </div>
  </div>
</template>
