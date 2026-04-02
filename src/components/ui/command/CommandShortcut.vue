<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/utils'

const props = defineProps<{
    class?: HTMLAttributes['class']
}>()
</script>

<template>
    <span
        data-slot="command-shortcut"
        :class="
            cn(
                'text-neutral-500 ml-auto text-xs tracking-widest dark:text-neutral-400',
                props.class
            )
        "
    >
        <slot />
    </span>
</template>
