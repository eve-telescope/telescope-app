<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/utils'

const props = defineProps<{
    class?: HTMLAttributes['class']
}>()
</script>

<template>
    <tr
        data-slot="table-row"
        :class="
            cn(
                'hover:bg-eve-bg-hover/50 data-[state=selected]:bg-eve-bg-3 border-b transition-colors',
                props.class
            )
        "
    >
        <slot />
    </tr>
</template>
