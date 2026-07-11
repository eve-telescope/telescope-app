import { createStore } from '@tauri-store/vue'
import { computed } from 'vue'

export interface Settings {
    [key: string]: unknown
    globalShortcut: string
    autoScanOnShortcut: boolean
    sortColumn: string
    sortDirection: 'asc' | 'desc'
    overlayLocked: boolean
}

export const DEFAULT_SETTINGS: Settings = {
    globalShortcut: 'CommandOrControl+Shift+V',
    autoScanOnShortcut: true,
    sortColumn: 'threat',
    sortDirection: 'desc',
    overlayLocked: false,
}

export const settingsStore = createStore<Settings>('settings', {
    ...DEFAULT_SETTINGS,
})

const startPromise = settingsStore.$tauri.start()

export function useSettings() {
    const settings = computed(() => settingsStore.value)
    const loaded = computed(() => true)

    async function updateSetting<K extends keyof Settings>(
        key: K,
        value: Settings[K]
    ) {
        await startPromise
        settingsStore.value = { ...settingsStore.value, [key]: value }
    }

    async function resetSettings() {
        await startPromise
        settingsStore.value = { ...DEFAULT_SETTINGS }
    }

    return {
        settings,
        loaded,
        updateSetting,
        resetSettings,
        settingsStore,
    }
}
