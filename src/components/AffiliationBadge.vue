<script setup lang="ts">
import { computed } from 'vue'
import { getAllianceLogoUrl, getCorporationLogoUrl } from '../utils/format'

/**
 * Logo + name for a corporation or alliance, in one row.
 * Renders an em-dash when the entity is unknown.
 */
const props = defineProps<{
    type: 'corporation' | 'alliance'
    entityId?: number | null
    name?: string | null
}>()

const logoUrl = computed(() => {
    if (!props.entityId) return null
    return props.type === 'corporation'
        ? getCorporationLogoUrl(props.entityId)
        : getAllianceLogoUrl(props.entityId)
})
</script>

<template>
    <div
        class="flex items-center gap-1.5 text-xs text-eve-text-2 min-w-0"
        :title="name || ''"
    >
        <img
            v-if="logoUrl"
            :src="logoUrl"
            :alt="name || ''"
            class="w-4 h-4 rounded-xs shrink-0"
            loading="lazy"
        />
        <span class="truncate">{{ name || '—' }}</span>
    </div>
</template>
