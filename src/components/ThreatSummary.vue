<script setup lang="ts">
import { computed } from "vue";
import { AlertTriangle } from "lucide-vue-next";
import type { PilotIntel } from "../types";

const props = defineProps<{
  pilots: PilotIntel[];
}>();

const threatCounts = computed(() => {
  const counts: Record<string, number> = {
    extreme: 0,
    high: 0,
    moderate: 0,
    low: 0,
    minimal: 0,
    unknown: 0,
  };
  
  for (const pilot of props.pilots) {
    const level = pilot.threat_level.toLowerCase();
    if (level in counts) {
      counts[level]++;
    } else {
      counts.unknown++;
    }
  }
  
  return counts;
});

const hasDangerousPilots = computed(() => {
  return threatCounts.value.extreme > 0 || threatCounts.value.high > 0;
});

const dangerousCount = computed(() => {
  return threatCounts.value.extreme + threatCounts.value.high;
});
</script>

<template>
  <div class="space-y-2">
    <!-- Threat Counts -->
    <div class="space-y-1">
      <div class="flex items-center justify-between text-xs" v-if="threatCounts.extreme > 0">
        <span class="flex items-center gap-1.5">
          <span class="w-2 h-2 rounded-full bg-eve-threat-extreme"></span>
          <span class="text-eve-text-2">Extreme</span>
        </span>
        <span class="font-mono text-eve-threat-extreme font-semibold">{{ threatCounts.extreme }}</span>
      </div>
      
      <div class="flex items-center justify-between text-xs" v-if="threatCounts.high > 0">
        <span class="flex items-center gap-1.5">
          <span class="w-2 h-2 rounded-full bg-eve-threat-high"></span>
          <span class="text-eve-text-2">High</span>
        </span>
        <span class="font-mono text-eve-threat-high font-semibold">{{ threatCounts.high }}</span>
      </div>
      
      <div class="flex items-center justify-between text-xs" v-if="threatCounts.moderate > 0">
        <span class="flex items-center gap-1.5">
          <span class="w-2 h-2 rounded-full bg-eve-threat-moderate"></span>
          <span class="text-eve-text-2">Moderate</span>
        </span>
        <span class="font-mono text-eve-threat-moderate">{{ threatCounts.moderate }}</span>
      </div>
      
      <div class="flex items-center justify-between text-xs" v-if="threatCounts.low > 0">
        <span class="flex items-center gap-1.5">
          <span class="w-2 h-2 rounded-full bg-eve-threat-low"></span>
          <span class="text-eve-text-2">Low</span>
        </span>
        <span class="font-mono text-eve-threat-low">{{ threatCounts.low }}</span>
      </div>
      
      <div class="flex items-center justify-between text-xs" v-if="threatCounts.minimal > 0 || threatCounts.unknown > 0">
        <span class="flex items-center gap-1.5">
          <span class="w-2 h-2 rounded-full bg-eve-threat-minimal"></span>
          <span class="text-eve-text-2">Minimal/Unknown</span>
        </span>
        <span class="font-mono text-eve-text-3">{{ threatCounts.minimal + threatCounts.unknown }}</span>
      </div>
    </div>
    
    <!-- Alert Banner -->
    <div 
      v-if="hasDangerousPilots" 
      class="px-2 py-1.5 bg-eve-threat-extreme/10 border border-eve-threat-extreme/30 rounded text-[10px] text-eve-threat-extreme flex items-center gap-1.5"
    >
      <AlertTriangle class="w-3 h-3 shrink-0" />
      <span>{{ dangerousCount }} dangerous pilot{{ dangerousCount > 1 ? 's' : '' }} detected</span>
    </div>
  </div>
</template>
