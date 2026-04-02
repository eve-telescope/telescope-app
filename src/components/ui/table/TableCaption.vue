<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/utils'

const props = defineProps<{
    class?: HTMLAttributes['class']
}>()
</script>

<template>
    <caption
        data-slot="table-caption"
        :class="
            cn(
                'text-neutral-500 mt-4 text-sm dark:text-neutral-400',
                props.class
            )
        "
    >
        <slot />
    </caption>
</template>
