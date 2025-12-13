<script setup lang="ts">
import { ChevronUp, ChevronDown } from "lucide-vue-next";

defineProps<{
  label?: string;
  sortKey: string;
  currentSort: string;
  sortDirection: "asc" | "desc";
  align?: "left" | "right";
}>();

const emit = defineEmits<{
  sort: [key: string];
}>();
</script>

<template>
  <button
    class="flex items-center gap-1 hover:text-eve-text-1 transition-colors cursor-pointer select-none"
    :class="[
      align === 'right' ? 'justify-end ml-auto' : '',
      currentSort === sortKey ? 'text-eve-cyan' : ''
    ]"
    @click="emit('sort', sortKey)"
  >
    <slot>{{ label }}</slot>
    <span v-if="currentSort === sortKey" class="w-3 h-3">
      <ChevronUp v-if="sortDirection === 'asc'" class="w-3 h-3" />
      <ChevronDown v-else class="w-3 h-3" />
    </span>
    <span v-else class="w-3 h-3 opacity-0 group-hover:opacity-30">
      <ChevronUp class="w-3 h-3" />
    </span>
  </button>
</template>

