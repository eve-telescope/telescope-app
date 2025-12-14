import { onUnmounted, watch, onMounted } from 'vue'
import {
    register,
    unregister,
    isRegistered,
} from '@tauri-apps/plugin-global-shortcut'
import { readText } from '@tauri-apps/plugin-clipboard-manager'
import { useSettings } from './useSettings'
import { info } from '@tauri-apps/plugin-log'

export function useGlobalShortcut(onPaste: (text: string) => void) {
    const { settings } = useSettings()
    const registerShortcut = async (shortcut: string) => {
        await unregisterShortcut(shortcut)
        await register(shortcut, async (event) => {
            if (event.state === 'Pressed') {
                info(`Global shortcut triggered: ${shortcut}`)
                onPaste(await readText())
            }
        })
    }

    const unregisterShortcut = async (shortcut: string) => {
        if (await isRegistered(shortcut)) {
            await unregister(shortcut)
        }
    }

    onMounted(() => registerShortcut(settings.value.globalShortcut))
    onUnmounted(() => unregisterShortcut(settings.value.globalShortcut))

    watch(
        () => settings.value.globalShortcut,
        async (newShortcut, oldShortcut) => {
            if (oldShortcut) {
                await unregisterShortcut(oldShortcut)
            }
            await registerShortcut(newShortcut)
        }
    )

    return {
        displayShortcut: () => settings.value.globalShortcut,
        updateShortcut: (shortcut: string) => {
            settings.value.globalShortcut = shortcut
        },
    }
}
