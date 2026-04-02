<script setup lang="ts">
import type { ContextMenuItemEmits, ContextMenuItemProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { ContextMenuItem, useForwardPropsEmits } from 'reka-ui'
import { cn } from '@/lib/utils'

const props = withDefaults(
    defineProps<
        ContextMenuItemProps & {
            class?: HTMLAttributes['class']
            inset?: boolean
            variant?: 'default' | 'destructive'
        }
    >(),
    {
        variant: 'default',
    }
)
const emits = defineEmits<ContextMenuItemEmits>()

const delegatedProps = reactiveOmit(props, 'class')

const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
    <ContextMenuItem
        data-slot="context-menu-item"
        :data-inset="inset ? '' : undefined"
        :data-variant="variant"
        v-bind="forwarded"
        :class="
            cn(
                'relative flex cursor-default items-center gap-2 rounded-sm px-2 py-1.5 text-xs outline-hidden select-none data-[disabled]:pointer-events-none data-[disabled]:opacity-50 data-[inset]:pl-8 focus:bg-eve-bg-3 focus:text-eve-text-1 data-[variant=destructive]:text-eve-red data-[variant=destructive]:focus:bg-eve-red/10 [&_svg:not([class*=\'text-\'])]:text-eve-text-3 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*=\'size-\'])]:size-4',
                props.class
            )
        "
    >
        <slot />
    </ContextMenuItem>
</template>
