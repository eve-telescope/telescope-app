<script setup lang="ts">
import { ref } from 'vue'
import { Share2, Loader2, Check, AlertCircle } from 'lucide-vue-next'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { createShare } from '../utils/share'

const props = defineProps<{
    pilotNames: string
}>()

const state = ref<'idle' | 'loading' | 'copied' | 'error'>('idle')

async function share() {
    if (state.value === 'loading') return

    state.value = 'loading'
    try {
        const result = await createShare(props.pilotNames)
        await writeText(result.url)
        state.value = 'copied'
        setTimeout(() => {
            state.value = 'idle'
        }, 2000)
    } catch (e) {
        console.error('Failed to create share:', e)
        state.value = 'error'
        setTimeout(() => {
            state.value = 'idle'
        }, 3000)
    }
}
</script>

<template>
    <button
        class="w-full py-2 bg-eve-bg-2 border border-eve-border rounded text-eve-text-2 text-xs font-medium tracking-wider cursor-pointer transition-all hover:border-eve-cyan-dim hover:text-eve-cyan disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
        :class="{
            'border-green-500/50 text-green-400': state === 'copied',
            'border-red-500/50 text-red-400': state === 'error',
        }"
        :disabled="state === 'loading'"
        @click="share"
    >
        <Loader2 v-if="state === 'loading'" class="w-3.5 h-3.5 animate-spin" />
        <Check v-else-if="state === 'copied'" class="w-3.5 h-3.5" />
        <AlertCircle v-else-if="state === 'error'" class="w-3.5 h-3.5" />
        <Share2 v-else class="w-3.5 h-3.5" />
        <template v-if="state === 'loading'">CREATING LINK...</template>
        <template v-else-if="state === 'copied'">LINK COPIED</template>
        <template v-else-if="state === 'error'">SHARE FAILED</template>
        <template v-else>SHARE SCAN</template>
    </button>
</template>
