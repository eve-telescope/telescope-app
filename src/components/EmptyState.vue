<script setup lang="ts">
import { Radar } from "lucide-vue-next";
</script>

<template>
  <div class="flex-1 flex flex-col items-center justify-center gap-4 text-eve-text-3">
    <Radar class="w-16 h-16 opacity-20" :stroke-width="1" />
    <div class="text-center">
      <p class="text-sm text-eve-text-2 mb-1">No pilots scanned</p>
      <p class="text-xs">Paste names from local chat to begin</p>
    </div>
  </div>
</template>
