<script setup lang="ts">
import { type HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import {
    ScrollAreaScrollbar,
    type ScrollAreaScrollbarProps,
    ScrollAreaThumb,
} from 'reka-ui'
import { cn } from '@/lib/utils'

const props = withDefaults(
    defineProps<
        ScrollAreaScrollbarProps & { class?: HTMLAttributes['class'] }
    >(),
    { orientation: 'vertical' }
)

const delegatedProps = reactiveOmit(props, 'class')
</script>

<template>
    <ScrollAreaScrollbar
        v-bind="delegatedProps"
        :class="
            cn(
                'flex touch-none select-none transition-colors',
                orientation === 'vertical' &&
                    'h-full w-2 border-l border-l-transparent p-px',
                orientation === 'horizontal' &&
                    'h-2 flex-col border-t border-t-transparent p-px',
                props.class
            )
        "
    >
        <ScrollAreaThumb
            class="relative flex-1 rounded-full bg-eve-bg-3 hover:bg-eve-border"
        />
    </ScrollAreaScrollbar>
</template>
