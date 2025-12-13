<script setup lang="ts">
import { computed } from "vue";
import type { ActivityHeatmap as ActivityHeatmapType } from "../types";

const props = defineProps<{
  activity: ActivityHeatmapType;
}>();

const DAYS = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

const eveTime = computed(() => {
  const now = new Date();
  return {
    day: now.getUTCDay(),
    hour: now.getUTCHours(),
    minutes: now.getUTCMinutes(),
  };
});

const currentHourPercent = computed(() => {
  return ((eveTime.value.hour + eveTime.value.minutes / 60) / 24) * 100;
});

const currentDayPercent = computed(() => {
  return ((eveTime.value.day + 0.5) / 7) * 100;
});

function getHeatColor(value: number): string {
  if (value === 0) return "transparent";
  const intensity = Math.min(value / props.activity.max, 1);
  const alpha = 0.2 + intensity * 0.8;
  return `rgba(68, 221, 170, ${alpha})`;
}

function formatHour(hour: number): string {
  return hour.toString().padStart(2, "0");
}
</script>

<template>
  <div class="flex flex-col gap-0.5">
    <!-- Hour labels -->
    <div class="flex items-center mb-0.5">
      <span class="w-7"></span>
      <div class="flex-1 flex justify-between text-[8px] text-eve-text-3 px-0.5">
        <span>00</span>
        <span>06</span>
        <span>12</span>
        <span>18</span>
        <span>23</span>
      </div>
    </div>
    
    <!-- Heatmap rows with crosshair -->
    <div class="relative">
      <!-- Vertical line (current hour) -->
      <div 
        class="absolute top-0 bottom-0 w-px bg-eve-orange/30 z-10 pointer-events-none"
        :style="{ left: `calc(28px + (100% - 28px) * ${currentHourPercent / 100})` }"
      ></div>
      
      <!-- Horizontal line (current day) -->
      <div 
        class="absolute left-7 right-0 h-px bg-eve-orange/30 z-10 pointer-events-none"
        :style="{ top: `${currentDayPercent}%` }"
      ></div>
      
      <!-- Rows -->
      <div
        v-for="(dayData, dayIndex) in activity.data"
        :key="dayIndex"
        class="flex items-center gap-1"
      >
        <span 
          class="text-[9px] w-6 shrink-0"
          :class="dayIndex === eveTime.day ? 'text-eve-cyan font-semibold' : 'text-eve-text-3'"
        >{{ DAYS[dayIndex] }}</span>
        <div class="flex gap-px flex-1">
          <div
            v-for="(value, hour) in dayData"
            :key="hour"
            class="flex-1 h-2.5 rounded-sm transition-transform hover:scale-y-150 hover:z-20"
            :class="value === 0 ? 'bg-eve-bg-3' : ''"
            :style="{ background: getHeatColor(value) }"
            :title="`${DAYS[dayIndex]} ${formatHour(hour as number)}:00 - ${value} kills`"
          ></div>
        </div>
      </div>
    </div>
    
    <!-- Current time display -->
    <div class="text-right mt-1">
      <span class="font-mono text-eve-cyan text-[9px]">
        {{ formatHour(eveTime.hour) }}:{{ formatHour(eveTime.minutes) }} EVE
      </span>
    </div>
  </div>
</template>

