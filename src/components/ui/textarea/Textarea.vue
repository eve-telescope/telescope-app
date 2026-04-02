<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { useVModel } from '@vueuse/core'
import { cn } from '@/lib/utils'

const props = defineProps<{
    class?: HTMLAttributes['class']
    defaultValue?: string | number
    modelValue?: string | number
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
    <textarea
        v-model="modelValue"
        data-slot="textarea"
        :class="
            cn(
                'flex field-sizing-content min-h-20 w-full rounded-md border border-eve-border bg-eve-bg-2 px-3 py-2 text-[11px] leading-relaxed text-eve-text-1 shadow-xs outline-none transition-[color,box-shadow] placeholder:text-eve-text-3 focus-visible:border-eve-cyan focus-visible:ring-[3px] focus-visible:ring-eve-cyan/25 aria-invalid:border-eve-red aria-invalid:ring-eve-red/20 disabled:cursor-not-allowed disabled:opacity-50',
                props.class
            )
        "
    />
</template>
