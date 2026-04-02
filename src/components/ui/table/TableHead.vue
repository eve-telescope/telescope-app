<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/utils'

const props = defineProps<{
    class?: HTMLAttributes['class']
}>()
</script>

<template>
    <th
        data-slot="table-head"
        :class="
            cn(
                'text-neutral-950 h-10 px-2 text-left align-middle font-medium whitespace-nowrap [&:has([role=checkbox])]:pr-0 [&>[role=checkbox]]:translate-y-[2px] dark:text-neutral-50',
                props.class
            )
        "
    >
        <slot />
    </th>
</template>
