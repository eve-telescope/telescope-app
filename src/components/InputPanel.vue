<script setup lang="ts">
import { ref } from "vue";
import { X, Loader2, Share2, Check, AlertCircle } from "lucide-vue-next";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import type { PilotIntel } from "../types";
import ThreatSummary from "./ThreatSummary.vue";
import ShortcutEditor from "./ShortcutEditor.vue";
import { createShare } from "../utils/share";

defineProps<{
  loading: boolean;
  pilotCount: number;
  pilots: PilotIntel[];
  shortcut: string;
  shortcutActive: boolean;
  shortcutError?: string | null;
}>();

const pilotNames = defineModel<string>("pilotNames", { required: true });

const emit = defineEmits<{
  scan: [];
  clear: [];
  "update:shortcut": [shortcut: string];
}>();

const shareState = ref<"idle" | "loading" | "copied" | "error">("idle");

function handleKeydown(e: KeyboardEvent) {
  if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
    emit("scan");
  }
}

async function copyShareLink() {
  if (shareState.value === "loading") return;

  shareState.value = "loading";
  try {
    const result = await createShare(pilotNames.value);
    await writeText(result.url);
    shareState.value = "copied";
    setTimeout(() => {
      shareState.value = "idle";
    }, 2000);
  } catch (e) {
    console.error("Failed to create share:", e);
    shareState.value = "error";
    setTimeout(() => {
      shareState.value = "idle";
    }, 3000);
  }
}
</script>

<template>
  <aside class="w-64 bg-eve-bg-1 border-r border-eve-border flex flex-col shrink-0">
    <!-- Input Section -->
    <div class="p-3 border-b border-eve-border flex-1 flex flex-col min-h-0">
      <h3 class="text-[10px] font-semibold tracking-[0.15em] text-eve-text-3 mb-2 shrink-0">PASTE LOCAL</h3>
      <textarea
        v-model="pilotNames"
        placeholder="Pilot names..."
        :disabled="loading"
        @keydown="handleKeydown"
        class="w-full flex-1 min-h-32 px-2.5 py-2 bg-eve-bg-2 border border-eve-border rounded text-eve-text-1 font-mono text-[11px] leading-relaxed resize-none focus:outline-none focus:border-eve-cyan-dim placeholder:text-eve-text-3 disabled:opacity-50"
      ></textarea>
      
      <div class="flex gap-2 mt-2 shrink-0">
        <button
          class="flex-1 py-2 bg-eve-cyan rounded text-eve-bg-0 text-xs font-bold tracking-wider cursor-pointer transition-all hover:bg-eve-cyan-dim disabled:opacity-40 disabled:cursor-not-allowed flex items-center justify-center gap-2"
          :disabled="loading || !pilotNames.trim()"
          @click="emit('scan')"
        >
          <Loader2 v-if="loading" class="w-3.5 h-3.5 animate-spin" />
          <template v-else>
            SCAN
            <span v-if="pilotCount > 0" class="bg-black/20 px-1.5 py-0.5 rounded text-[10px]">{{ pilotCount }}</span>
          </template>
        </button>
        
        <button
          v-if="pilotNames.trim()"
          class="px-3 py-2 bg-transparent border border-eve-border rounded text-eve-text-3 cursor-pointer transition-colors hover:border-eve-text-3 hover:text-eve-text-1 flex items-center justify-center"
          @click="emit('clear')"
          title="Clear"
        >
          <X class="w-3.5 h-3.5" />
        </button>
      </div>
    </div>

    <!-- Threat Summary -->
    <div v-if="pilots.length > 0" class="p-3 shrink-0">
      <h3 class="text-[10px] font-semibold tracking-[0.15em] text-eve-text-3 mb-2">THREAT SUMMARY</h3>
      <ThreatSummary :pilots="pilots" />
    </div>

    <!-- Share Button -->
    <div v-if="pilots.length > 0" class="px-3 pb-3 shrink-0">
      <button
        class="w-full py-2 bg-eve-bg-2 border border-eve-border rounded text-eve-text-2 text-xs font-medium tracking-wider cursor-pointer transition-all hover:border-eve-cyan-dim hover:text-eve-cyan disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
        :class="{
          'border-green-500/50 text-green-400': shareState === 'copied',
          'border-red-500/50 text-red-400': shareState === 'error',
        }"
        :disabled="shareState === 'loading'"
        @click="copyShareLink"
      >
        <Loader2 v-if="shareState === 'loading'" class="w-3.5 h-3.5 animate-spin" />
        <Check v-else-if="shareState === 'copied'" class="w-3.5 h-3.5" />
        <AlertCircle v-else-if="shareState === 'error'" class="w-3.5 h-3.5" />
        <Share2 v-else class="w-3.5 h-3.5" />
        <template v-if="shareState === 'loading'">CREATING LINK...</template>
        <template v-else-if="shareState === 'copied'">LINK COPIED</template>
        <template v-else-if="shareState === 'error'">SHARE FAILED</template>
        <template v-else>SHARE SCAN</template>
      </button>
    </div>

    <!-- Shortcut Settings -->
    <div class="p-3 border-t border-eve-border shrink-0">
      <ShortcutEditor
        :shortcut="shortcut"
        :is-active="shortcutActive"
        :error="shortcutError"
        @update="emit('update:shortcut', $event)"
      />
      <p class="text-[9px] text-eve-text-3 opacity-50 mt-2">Data from zKillboard</p>
    </div>
  </aside>
</template>
