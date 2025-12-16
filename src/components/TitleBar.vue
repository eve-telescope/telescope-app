<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { platform } from '@tauri-apps/plugin-os'
import { Crosshair, Minus, Square, X, Copy, Layers } from 'lucide-vue-next'
import { useOverlayWindow } from '../composables/useOverlayWindow'

defineProps<{
    pilotCount: number
}>()

const appWindow = getCurrentWindow()
const isMac = ref(false)
const isMaximized = ref(false)

const { isOverlayOpen, toggleOverlay } = useOverlayWindow()

onMounted(async () => {
    isMac.value = platform() === 'macos'
    isMaximized.value = await appWindow.isMaximized()

    if (!isMac.value) {
        await appWindow.setDecorations(false)
    }

    appWindow.onResized(async () => {
        isMaximized.value = await appWindow.isMaximized()
    })
})

async function minimize() {
    await appWindow.minimize()
}

async function toggleMaximize() {
    await appWindow.toggleMaximize()
}

async function close() {
    await appWindow.close()
}
</script>

<template>
    <header
        data-tauri-drag-region
        class="titlebar h-11 flex items-center justify-between px-4 bg-eve-bg-1/80 backdrop-blur-sm border-b border-eve-border shrink-0"
    >
        <!-- Left: Spacer for macOS traffic lights -->
        <div class="w-20 shrink-0"></div>

        <!-- Center: Logo -->
        <div data-tauri-drag-region class="flex items-center gap-2">
            <Crosshair class="w-5 h-5 text-eve-cyan" :stroke-width="1.5" />
            <span class="text-xs font-bold tracking-[0.25em] text-eve-text-2"
                >TELESCOPE</span
            >
            <span
                v-if="pilotCount > 0"
                class="ml-2 px-1.5 py-0.5 bg-eve-cyan/10 rounded text-[10px] font-mono text-eve-cyan"
            >
                {{ pilotCount }}
            </span>
        </div>

        <!-- Right: Overlay toggle + Window controls -->
        <div class="shrink-0 flex items-center justify-end">
            <!-- Overlay toggle -->
            <button
                class="px-2 py-1 mr-2 flex items-center gap-1.5 rounded text-[10px] font-medium transition-all"
                :class="
                    isOverlayOpen
                        ? 'bg-eve-cyan/20 text-eve-cyan'
                        : 'text-eve-text-3 hover:text-eve-text-1 hover:bg-white/5'
                "
                @click="toggleOverlay"
                title="Toggle overlay window"
            >
                <Layers class="w-3.5 h-3.5" :stroke-width="1.5" />
                <span class="hidden sm:inline">Overlay</span>
            </button>

            <!-- Window controls (Windows/Linux only) -->
            <template v-if="!isMac">
                <button
                    class="w-11 h-8 flex items-center justify-center text-eve-text-3 hover:bg-white/10 hover:text-eve-text-1 transition-colors"
                    @click="minimize"
                    title="Minimize"
                >
                    <Minus class="w-4 h-4" :stroke-width="1.5" />
                </button>
                <button
                    class="w-11 h-8 flex items-center justify-center text-eve-text-3 hover:bg-white/10 hover:text-eve-text-1 transition-colors"
                    @click="toggleMaximize"
                    :title="isMaximized ? 'Restore' : 'Maximize'"
                >
                    <Copy
                        v-if="isMaximized"
                        class="w-3.5 h-3.5"
                        :stroke-width="1.5"
                    />
                    <Square v-else class="w-3 h-3" :stroke-width="1.5" />
                </button>
                <button
                    class="w-11 h-8 flex items-center justify-center text-eve-text-3 hover:bg-[#c42b1c] hover:text-white transition-colors"
                    @click="close"
                    title="Close"
                >
                    <X class="w-4 h-4" :stroke-width="1.5" />
                </button>
            </template>
        </div>
    </header>
</template>

<style>
[data-tauri-drag-region] {
    -webkit-user-select: none;
    user-select: none;
    -webkit-app-region: drag;
    app-region: drag;
}

[data-tauri-drag-region] button {
    -webkit-app-region: no-drag;
    app-region: no-drag;
}
</style>
