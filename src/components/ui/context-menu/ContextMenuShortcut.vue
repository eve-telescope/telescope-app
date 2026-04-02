<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/utils'

const props = defineProps<{
    class?: HTMLAttributes['class']
}>()
</script>

<template>
    <span
        data-slot="context-menu-shortcut"
        :class="
            cn(
                'ml-auto text-[10px] tracking-widest text-eve-text-3',
                props.class
            )
        "
    >
        <slot />
    </span>
</template>
