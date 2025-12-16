import { ref, watch } from 'vue'
import { LazyStore } from '@tauri-apps/plugin-store'
import { info, error as logError } from '@tauri-apps/plugin-log'

export interface Settings {
    globalShortcut: string
    autoScanOnShortcut: boolean
    sortColumn: string
    sortDirection: 'asc' | 'desc'
    overlayLocked: boolean
}

const DEFAULT_SETTINGS: Settings = {
    globalShortcut: 'CommandOrControl+Shift+V',
    autoScanOnShortcut: true,
    sortColumn: 'threat',
    sortDirection: 'desc',
    overlayLocked: false,
}

const store = new LazyStore('settings.json')

const settings = ref<Settings>({ ...DEFAULT_SETTINGS })
const loaded = ref(false)

export function useSettings() {
    async function loadSettings() {
        try {
            const stored = await store.get<Settings>('settings')
            if (stored) {
                settings.value = { ...DEFAULT_SETTINGS, ...stored }
                info('Settings loaded from store')
            } else {
                settings.value = { ...DEFAULT_SETTINGS }
                info('Using default settings')
            }
            loaded.value = true
        } catch (e) {
            logError(`Failed to load settings: ${e}`)
            settings.value = { ...DEFAULT_SETTINGS }
            loaded.value = true
        }
    }

    async function saveSettings() {
        try {
            await store.set('settings', settings.value)
            await store.save()
            info('Settings saved')
        } catch (e) {
            logError(`Failed to save settings: ${e}`)
        }
    }

    async function updateSetting<K extends keyof Settings>(
        key: K,
        value: Settings[K]
    ) {
        settings.value[key] = value
        await saveSettings()
    }

    async function resetSettings() {
        settings.value = { ...DEFAULT_SETTINGS }
        await saveSettings()
        info('Settings reset to defaults')
    }

    watch(
        settings,
        () => {
            if (loaded.value) {
                saveSettings()
            }
        },
        { deep: true }
    )

    if (!loaded.value) {
        loadSettings()
    }

    return {
        settings,
        loaded,
        loadSettings,
        saveSettings,
        updateSetting,
        resetSettings,
    }
}
