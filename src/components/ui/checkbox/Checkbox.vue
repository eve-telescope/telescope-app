<script setup lang="ts">
import type { CheckboxRootEmits, CheckboxRootProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { Check } from 'lucide-vue-next'
import { CheckboxIndicator, CheckboxRoot, useForwardPropsEmits } from 'reka-ui'
import { cn } from '@/lib/utils'

const props = defineProps<
    CheckboxRootProps & { class?: HTMLAttributes['class'] }
>()
const emits = defineEmits<CheckboxRootEmits>()

const delegatedProps = reactiveOmit(props, 'class')

const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
    <CheckboxRoot
        v-slot="slotProps"
        data-slot="checkbox"
        v-bind="forwarded"
        :class="
            cn(
                'peer size-4 shrink-0 rounded-[4px] border border-eve-border bg-eve-bg-2 text-eve-bg-0 shadow-xs outline-none transition-shadow data-[state=checked]:border-eve-cyan data-[state=checked]:bg-eve-cyan data-[state=checked]:text-eve-bg-0 focus-visible:ring-[3px] focus-visible:ring-eve-cyan/25 aria-invalid:border-eve-red aria-invalid:ring-eve-red/20 disabled:cursor-not-allowed disabled:opacity-50',
                props.class
            )
        "
    >
        <CheckboxIndicator
            data-slot="checkbox-indicator"
            class="grid place-content-center text-current transition-none"
        >
            <slot v-bind="slotProps">
                <Check class="size-3.5" />
            </slot>
        </CheckboxIndicator>
    </CheckboxRoot>
</template>
