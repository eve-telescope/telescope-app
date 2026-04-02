<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { useVModel } from '@vueuse/core'
import { cn } from '@/lib/utils'

const props = defineProps<{
    defaultValue?: string | number
    modelValue?: string | number
    class?: HTMLAttributes['class']
}>()

const emits = defineEmits<{
    (e: 'update:modelValue', payload: string | number): void
}>()

const modelValue = useVModel(props, 'modelValue', emits, {
    passive: true,
    defaultValue: props.defaultValue,
})
</script>

<template>
    <input
        v-model="modelValue"
        data-slot="input"
        :class="
            cn(
                'h-9 w-full min-w-0 rounded-md border border-eve-border bg-eve-bg-2 px-3 py-1 text-xs text-eve-text-1 shadow-xs outline-none transition-[color,box-shadow] placeholder:text-eve-text-3 selection:bg-eve-cyan selection:text-eve-bg-0 file:inline-flex file:h-7 file:border-0 file:bg-transparent file:text-xs file:font-medium file:text-eve-text-2 disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50',
                'focus-visible:border-eve-cyan focus-visible:ring-[3px] focus-visible:ring-eve-cyan/25',
                'aria-invalid:border-eve-red aria-invalid:ring-eve-red/20',
                props.class
            )
        "
    />
</template>
