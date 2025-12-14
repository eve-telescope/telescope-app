<script setup lang="ts">
import { computed, ref } from 'vue'
import { AlertTriangle, Check, Keyboard, RotateCcw, X } from 'lucide-vue-next'
import {
    formatShortcutKeys,
    keyEventToShortcut,
    validateShortcut,
} from '../utils/shortcut'

const props = defineProps<{
    shortcut: string
}>()

const emit = defineEmits<{
    update: [shortcut: string]
}>()

const isRecording = ref(false)
const recordedShortcut = ref<string | null>(null)

const displayKeys = computed(() => {
    if (isRecording.value && recordedShortcut.value) {
        return formatShortcutKeys(recordedShortcut.value)
    }
    return formatShortcutKeys(props.shortcut)
})

const validation = computed(() => {
    if (!recordedShortcut.value) return null
    return validateShortcut(recordedShortcut.value)
})

const canConfirm = computed(() => validation.value?.valid === true)
const hasConflict = computed(() => !!validation.value?.conflict)

function startRecording() {
    isRecording.value = true
    recordedShortcut.value = null
}

function stopRecording() {
    isRecording.value = false
    recordedShortcut.value = null
}

function confirmShortcut() {
    if (recordedShortcut.value && canConfirm.value) {
        emit('update', recordedShortcut.value)
    }
    stopRecording()
}

function handleKeydown(e: KeyboardEvent) {
    if (!isRecording.value) return

    e.preventDefault()
    e.stopPropagation()

    if (e.key === 'Escape') {
        stopRecording()
        return
    }

    const shortcut = keyEventToShortcut(e)
    if (shortcut) {
        recordedShortcut.value = shortcut
    }
}

function resetToDefault() {
    emit('update', 'CommandOrControl+Shift+V')
}
</script>

<template>
    <div class="space-y-2">
        <div class="flex items-center justify-between">
            <span
                class="text-[10px] font-semibold tracking-[0.15em] text-eve-text-3"
                >GLOBAL SHORTCUT</span
            >
            <button
                v-if="!isRecording"
                class="text-[9px] text-eve-text-3 hover:text-eve-text-2 transition-colors"
                @click="resetToDefault"
                title="Reset to default"
            >
                <RotateCcw class="w-3 h-3" />
            </button>
        </div>

        <!-- Shortcut display/editor -->
        <div
            class="relative flex items-center gap-3 px-3 py-2.5 rounded border transition-all cursor-pointer"
            :class="[
                isRecording
                    ? 'bg-eve-cyan/10 border-eve-cyan'
                    : 'bg-eve-bg-2 border-eve-border hover:border-eve-text-3',
            ]"
            tabindex="0"
            @click="startRecording"
            @keydown="handleKeydown"
            @blur="isRecording && !recordedShortcut ? stopRecording() : null"
        >
            <Keyboard
                class="w-4 h-4 shrink-0"
                :class="isRecording ? 'text-eve-cyan' : 'text-eve-text-3'"
            />

            <div class="flex-1 flex items-center gap-1.5 min-w-0">
                <template v-if="isRecording && !recordedShortcut">
                    <span class="text-sm text-eve-cyan animate-pulse"
                        >Press keys...</span
                    >
                </template>
                <template v-else>
                    <kbd
                        v-for="(key, i) in displayKeys"
                        :key="i"
                        class="px-2 py-1 rounded bg-eve-bg-3 text-sm font-medium min-w-7 text-center data-editing:border-eve-cyan/30 data-editing:text-eve-cyan data-editing:border"
                        :data-editing="
                            isRecording && recordedShortcut ? '' : undefined
                        "
                        >{{ key }}</kbd
                    >
                </template>
            </div>

            <!-- Status indicator -->
            <div class="flex items-center gap-1 shrink-0">
                <template v-if="isRecording && recordedShortcut">
                    <button
                        class="p-1.5 rounded transition-colors data-conflict:hover:bg-eve-orange/20 data-conflict:text-eve-orange hover:bg-eve-green/20 text-eve-green"
                        :data-conflict="hasConflict ? '' : undefined"
                        :disabled="!canConfirm"
                        @click.stop="confirmShortcut"
                        :title="hasConflict ? 'Use anyway' : 'Confirm'"
                    >
                        <AlertTriangle v-if="hasConflict" class="w-4 h-4" />
                        <Check v-else class="w-4 h-4" />
                    </button>
                    <button
                        class="p-1.5 rounded hover:bg-eve-red/20 text-eve-red transition-colors"
                        @click.stop="stopRecording"
                        title="Cancel"
                    >
                        <X class="w-4 h-4" />
                    </button>
                </template>
            </div>
        </div>

        <!-- Validation message -->
        <p
            v-if="isRecording && validation?.message"
            class="text-[10px]"
            :class="hasConflict ? 'text-eve-orange' : 'text-eve-red'"
        >
            {{ validation.message }}
        </p>

        <!-- Help text -->
        <p class="text-[9px] text-eve-text-3 opacity-70">
            {{ isRecording ? 'Press Escape to cancel' : 'Click to change' }}
        </p>
    </div>
</template>
