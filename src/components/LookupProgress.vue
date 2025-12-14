<script setup lang="ts">
import { computed } from 'vue'
import type { LookupProgress } from '../composables/usePilots'

const props = defineProps<{
    progress: LookupProgress
}>()

const percent = computed(() => {
    return Math.round((props.progress.current / props.progress.total) * 100)
})
</script>

<template>
    <div class="space-y-1">
        <div class="flex justify-between text-[9px] text-eve-text-3">
            <span v-if="progress.cache_hits > 0" class="text-green-400">
                {{ progress.cache_hits }} cached
            </span>
            <span v-else>Fetching...</span>
            <span>{{ progress.current }}/{{ progress.total }}</span>
        </div>
        <div class="h-1.5 bg-eve-bg-2 rounded-full overflow-hidden">
            <div
                class="h-full bg-eve-cyan transition-all duration-100 ease-out"
                :style="{ width: `${percent}%` }"
            />
        </div>
    </div>
</template>
