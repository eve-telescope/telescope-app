<script setup lang="ts">
import { openUrl } from "@tauri-apps/plugin-opener";
import type { SystemStats } from "../types";

const props = defineProps<{
  systems: SystemStats[];
  characterId: number;
  maxDisplay?: number;
}>();

function openSystemUrl(systemId: number, type: 'kills' | 'losses' | 'all'): void {
  const suffix = type === 'all' ? '' : `${type}/`;
  openUrl(`https://zkillboard.com/character/${props.characterId}/system/${systemId}/${suffix}`);
}
</script>

<template>
  <div v-if="systems.length" class="space-y-1">
    <div
      v-for="sys in systems.slice(0, maxDisplay || 5)"
      :key="sys.system_id"
      class="flex items-center gap-2 text-xs group relative"
    >
      <span class="text-eve-text-1 truncate max-w-24">{{ sys.system_name }}</span>
      <span class="font-mono text-eve-green text-[11px]">{{ sys.kills }}</span>
      
      <!-- Hover links -->
      <div class="absolute right-0 opacity-0 group-hover:opacity-100 transition-opacity flex gap-0.5 bg-eve-bg-2 pl-2">
        <button
          class="px-1 py-0.5 rounded bg-eve-bg-3 text-eve-green text-[9px] font-mono hover:bg-eve-green/20 transition-colors"
          @click.stop="openSystemUrl(sys.system_id, 'kills')"
          title="View kills"
        >K</button>
        <button
          class="px-1 py-0.5 rounded bg-eve-bg-3 text-eve-red text-[9px] font-mono hover:bg-eve-red/20 transition-colors"
          @click.stop="openSystemUrl(sys.system_id, 'losses')"
          title="View losses"
        >L</button>
        <button
          class="px-1 py-0.5 rounded bg-eve-bg-3 text-eve-cyan text-[9px] font-mono hover:bg-eve-cyan/20 transition-colors"
          @click.stop="openSystemUrl(sys.system_id, 'all')"
          title="View all"
        >B</button>
      </div>
    </div>
  </div>
  <div v-else class="text-xs text-eve-text-3 italic">No system data</div>
</template>
