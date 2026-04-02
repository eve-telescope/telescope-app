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
                'hover:bg-neutral-100/50 data-[state=selected]:bg-neutral-100 border-b transition-colors dark:hover:bg-neutral-800/50 dark:data-[state=selected]:bg-neutral-800',
                props.class
            )
        "
    >
        <slot />
    </tr>
</template>
