<script setup lang="ts">
import type { TabsListProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { TabsList } from 'reka-ui'
import { cn } from '@/lib/utils'

const props = defineProps<TabsListProps & { class?: HTMLAttributes['class'] }>()

const delegatedProps = reactiveOmit(props, 'class')
</script>

<template>
    <TabsList
        data-slot="tabs-list"
        v-bind="delegatedProps"
        :class="
            cn(
                'inline-flex h-9 w-fit items-center justify-center rounded-md border border-eve-border bg-eve-bg-2 p-[3px] text-eve-text-3',
                props.class
            )
        "
    >
        <slot />
    </TabsList>
</template>
