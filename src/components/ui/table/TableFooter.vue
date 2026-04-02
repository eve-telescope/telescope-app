<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/utils'

const props = defineProps<{
    class?: HTMLAttributes['class']
}>()
</script>

<template>
    <tfoot
        data-slot="table-footer"
        :class="
            cn(
                'bg-neutral-100/50 border-t font-medium [&>tr]:last:border-b-0 dark:bg-neutral-800/50',
                props.class
            )
        "
    >
        <slot />
    </tfoot>
</template>
