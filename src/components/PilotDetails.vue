<script setup lang="ts">
import type { PilotIntel, ZkillStats } from "../types";
import CombatStats from "./CombatStats.vue";
import ShipsList from "./ShipsList.vue";
import SystemsList from "./SystemsList.vue";
import ActivityHeatmap from "./ActivityHeatmap.vue";

defineProps<{
  pilot: PilotIntel;
  zkill: ZkillStats;
}>();
</script>

<template>
  <div class="px-4 py-3 pl-14 bg-eve-bg-2 border-t border-eve-border details-enter">
    <div class="grid grid-cols-4 gap-6">
      <!-- Combat Stats -->
      <div>
        <h5 class="text-[10px] font-semibold tracking-wider text-eve-text-3 uppercase mb-2 pb-1 border-b border-eve-border">Combat</h5>
        <CombatStats :stats="zkill" :character-id="pilot.character.id" />
      </div>

      <!-- Ships -->
      <div>
        <h5 class="text-[10px] font-semibold tracking-wider text-eve-text-3 uppercase mb-2 pb-1 border-b border-eve-border">Ships</h5>
        <ShipsList :ships="zkill.top_ships" :character-id="pilot.character.id" />
      </div>

      <!-- Systems -->
      <div>
        <h5 class="text-[10px] font-semibold tracking-wider text-eve-text-3 uppercase mb-2 pb-1 border-b border-eve-border">Systems</h5>
        <SystemsList :systems="zkill.top_systems" :character-id="pilot.character.id" :max-display="5" />
      </div>

      <!-- Activity Heatmap -->
      <div>
        <h5 class="text-[10px] font-semibold tracking-wider text-eve-text-3 uppercase mb-2 pb-1 border-b border-eve-border">Active Hours</h5>
        <ActivityHeatmap v-if="zkill.activity" :activity="zkill.activity" />
        <div v-else class="text-xs text-eve-text-3 italic">No activity data</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.details-enter {
  animation: details-slide 0.2s ease-out;
}

@keyframes details-slide {
  from {
    opacity: 0;
    transform: translateY(-8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
