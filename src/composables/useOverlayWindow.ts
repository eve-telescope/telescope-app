import { ref } from 'vue'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { emit, listen, type UnlistenFn } from '@tauri-apps/api/event'

let overlayWindow: WebviewWindow | null = null
const isOverlayOpen = ref(false)
let unlistenCloseRequest: UnlistenFn | null = null

export function useOverlayWindow() {
    async function openOverlay() {
        if (overlayWindow) {
            await overlayWindow.setFocus()
            return
        }

        overlayWindow = new WebviewWindow('overlay', {
            url: '/overlay',
            title: 'Telescope Overlay',
            width: 580,
            height: 450,
            minWidth: 480,
            minHeight: 200,
            resizable: true,
            decorations: false,
            transparent: true,
            alwaysOnTop: true,
            skipTaskbar: true,
            shadow: false,
            x: 100,
            y: 100,
        })

        overlayWindow.once('tauri://created', () => {
            isOverlayOpen.value = true
        })

        overlayWindow.once('tauri://destroyed', () => {
            overlayWindow = null
            isOverlayOpen.value = false
        })

        // Listen for close request from overlay
        unlistenCloseRequest = await listen('overlay-close-request', () => {
            closeOverlay()
        })
    }

    async function closeOverlay() {
        if (unlistenCloseRequest) {
            unlistenCloseRequest()
            unlistenCloseRequest = null
        }
        if (overlayWindow) {
            try {
                await overlayWindow.close()
            } catch (e) {
                // Window might already be closed
                console.log('Overlay close error (may be expected):', e)
            }
            overlayWindow = null
            isOverlayOpen.value = false
        }
    }

    async function toggleOverlay() {
        if (overlayWindow) {
            await closeOverlay()
        } else {
            await openOverlay()
        }
    }

    async function clearOverlay() {
        await emit('overlay-clear')
    }

    return {
        isOverlayOpen,
        openOverlay,
        closeOverlay,
        toggleOverlay,
        clearOverlay,
    }
}
