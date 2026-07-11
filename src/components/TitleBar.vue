<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useDebounceFn } from '@vueuse/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { platform } from '@tauri-apps/plugin-os'
import { Crosshair, Minus, Square, X, Copy, Layers } from 'lucide-vue-next'
import { useOverlayWindow } from '../composables/useOverlayWindow'
defineProps<{
    pilotCount: number
    activeTab: string
}>()

const emit = defineEmits<{
    'update:activeTab': [value: string]
}>()

const navItems = [
    { value: 'local', label: 'Local' },
    { value: 'dscan', label: 'D-Scan' },
    { value: 'networks', label: 'Network' },
    { value: 'settings', label: 'Settings' },
]

const appWindow = getCurrentWindow()
const isMac = ref(false)
const isMaximized = ref(false)

const { isOverlayOpen, toggleOverlay, closeOverlay, setupListeners, cleanup } =
    useOverlayWindow()

onMounted(async () => {
    isMac.value = platform() === 'macos'
    isMaximized.value = await appWindow.isMaximized()

    if (!isMac.value) {
        await appWindow.setDecorations(false)
    }

    // Resize events fire continuously during a drag-resize; debounce so the
    // isMaximized IPC round-trip runs once per gesture settle.
    const syncMaximized = useDebounceFn(async () => {
        isMaximized.value = await appWindow.isMaximized()
    }, 150)
    appWindow.onResized(() => {
        syncMaximized()
    })

    appWindow.onCloseRequested(async () => {
        try {
            await closeOverlay()
        } catch (e) {
            console.log('Error closing overlay:', e)
        }
    })

    await setupListeners()
})

onUnmounted(() => {
    cleanup()
})

async function minimize() {
    await appWindow.minimize()
}

async function toggleMaximize() {
    await appWindow.toggleMaximize()
}

async function close() {
    await closeOverlay()
    await appWindow.close()
}
</script>

<template>
    <header
        data-tauri-drag-region
        class="titlebar h-10 flex items-center justify-between px-3 bg-eve-bg-1/80 backdrop-blur-sm border-b border-eve-border shrink-0"
    >
        <!-- Left: Logo + pilot count -->
        <div
            class="flex items-center gap-2 shrink-0"
            :class="isMac ? 'ml-20' : 'ml-3'"
        >
            <Crosshair class="w-4 h-4 text-eve-cyan" :stroke-width="1.5" />
            <span class="text-[10px] font-bold tracking-[0.2em] text-eve-text-3"
                >TELESCOPE</span
            >
            <span
                v-if="pilotCount > 0"
                class="px-1.5 py-0.5 bg-eve-cyan/10 rounded text-[9px] font-mono text-eve-cyan"
            >
                {{ pilotCount }}
            </span>
        </div>

        <!-- Center: Navigation -->
        <nav data-tauri-drag-region class="flex-1 flex justify-center">
            <div class="flex items-center gap-1">
                <button
                    v-for="item in navItems"
                    :key="item.value"
                    class="px-3 py-1 rounded text-[10px] font-medium tracking-wider transition-colors"
                    :class="
                        activeTab === item.value
                            ? 'bg-eve-bg-3 text-eve-text-1'
                            : 'text-eve-text-3 hover:text-eve-text-1 hover:bg-eve-bg-hover'
                    "
                    @click="emit('update:activeTab', item.value)"
                >
                    {{ item.label }}
                </button>
            </div>
        </nav>

        <!-- Right: Overlay toggle + Window controls -->
        <div class="shrink-0 flex items-center justify-end">
            <button
                class="px-2 py-1 mr-1 flex items-center gap-1.5 rounded text-[10px] font-medium transition-all"
                :class="
                    isOverlayOpen
                        ? 'bg-eve-cyan/20 text-eve-cyan'
                        : 'text-eve-text-3 hover:text-eve-text-1 hover:bg-eve-bg-hover'
                "
                @click="toggleOverlay"
                title="Toggle overlay window"
            >
                <Layers class="w-3.5 h-3.5" :stroke-width="1.5" />
                <span class="hidden sm:inline">Overlay</span>
            </button>

            <template v-if="!isMac">
                <button
                    class="w-10 h-8 flex items-center justify-center text-eve-text-3 hover:bg-eve-bg-hover hover:text-eve-text-1 transition-colors"
                    @click="minimize"
                    title="Minimize"
                >
                    <Minus class="w-4 h-4" :stroke-width="1.5" />
                </button>
                <button
                    class="w-10 h-8 flex items-center justify-center text-eve-text-3 hover:bg-eve-bg-hover hover:text-eve-text-1 transition-colors"
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
                    class="w-10 h-8 flex items-center justify-center text-eve-text-3 hover:bg-eve-win-close hover:text-eve-text-1 transition-colors"
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
