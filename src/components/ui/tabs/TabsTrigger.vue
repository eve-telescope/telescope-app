<script setup lang="ts">
import type { TabsTriggerProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { TabsTrigger, useForwardProps } from 'reka-ui'
import { cn } from '@/lib/utils'

const props = defineProps<
    TabsTriggerProps & { class?: HTMLAttributes['class'] }
>()

const delegatedProps = reactiveOmit(props, 'class')

const forwardedProps = useForwardProps(delegatedProps)
</script>

<template>
    <TabsTrigger
        data-slot="tabs-trigger"
        :class="
            cn(
                'inline-flex h-[calc(100%-1px)] flex-1 items-center justify-center gap-1.5 rounded-sm border border-transparent px-2 py-1 text-[10px] font-semibold tracking-wider whitespace-nowrap text-eve-text-3 transition-[color,box-shadow] focus-visible:outline-1 focus-visible:ring-[3px] focus-visible:ring-eve-cyan/25 disabled:pointer-events-none disabled:opacity-50 data-[state=active]:border-eve-cyan/25 data-[state=active]:bg-eve-bg-3 data-[state=active]:text-eve-cyan data-[state=active]:shadow-sm [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*=\'size-\'])]:size-4',
                props.class
            )
        "
        v-bind="forwardedProps"
    >
        <slot />
    </TabsTrigger>
</template>
