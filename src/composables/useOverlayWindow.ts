import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { emit, listen, type UnlistenFn } from '@tauri-apps/api/event'

const isOverlayOpen = ref(false)
let unlistenClosed: UnlistenFn | null = null

export function useOverlayWindow() {
    async function syncState() {
        isOverlayOpen.value = await invoke<boolean>('is_overlay_open')
    }

    async function openOverlay() {
        isOverlayOpen.value = await invoke<boolean>('open_overlay')
    }

    async function closeOverlay() {
        isOverlayOpen.value = await invoke<boolean>('close_overlay')
    }

    async function toggleOverlay() {
        isOverlayOpen.value = await invoke<boolean>('toggle_overlay')
    }

    async function clearOverlay() {
        await emit('overlay-clear')
    }

    async function setupListeners() {
        // Listen for overlay being closed (from overlay's close button or elsewhere)
        unlistenClosed = await listen('overlay-closed', () => {
            isOverlayOpen.value = false
        })

        // Sync initial state
        await syncState()
    }

    function cleanup() {
        if (unlistenClosed) {
            unlistenClosed()
            unlistenClosed = null
        }
    }

    return {
        isOverlayOpen,
        openOverlay,
        closeOverlay,
        toggleOverlay,
        clearOverlay,
        setupListeners,
        cleanup,
    }
}
