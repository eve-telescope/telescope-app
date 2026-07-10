import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { info, error as logError } from '@tauri-apps/plugin-log'
import { API_BASE_URL } from '../utils/config'

interface ShareData {
    code: string
    pilots: string[]
    pilotCount: number
}

async function fetchShare(code: string): Promise<ShareData | null> {
    const url = `${API_BASE_URL}/api/share/${code}`
    try {
        await info(`[DeepLink] Fetching: ${url}`)
        const response = await fetch(url, {
            headers: { Accept: 'application/json' },
        })

        await info(`[DeepLink] Response status: ${response.status}`)

        if (!response.ok) {
            const text = await response.text()
            await logError(`[DeepLink] API error: ${response.status} - ${text}`)
            return null
        }

        return response.json()
    } catch (e) {
        await logError(`[DeepLink] Fetch failed: ${e}`)
        return null
    }
}

/**
 * Deep links are parsed and dispatched in Rust (src-tauri/src/deep_link.rs):
 * auth tokens are applied entirely backend-side and surface through the
 * regular intel-state-changed flow, so the frontend only handles share
 * codes — delivered via the 'deep-link-share' event while running, or
 * parked backend-side when the link arrived before the webview mounted.
 */
export function useDeepLink(onPilotsReceived: (pilots: string) => void) {
    const loading = ref(false)
    let unlisten: UnlistenFn | null = null

    async function loadShare(code: string) {
        loading.value = true
        try {
            await info(`[DeepLink] Loading share: ${code}`)
            const share = await fetchShare(code)
            if (share?.pilots) {
                await info(
                    `[DeepLink] Received ${share.pilots.length} pilots from share`
                )
                onPilotsReceived(share.pilots.join('\n'))
            }
        } catch (e) {
            await logError(`[DeepLink] Error fetching share: ${e}`)
        } finally {
            loading.value = false
        }
    }

    onMounted(async () => {
        unlisten = await listen<string>('deep-link-share', (event) => {
            void loadShare(event.payload)
        })

        // Collect a share link that launched the app before we mounted.
        try {
            const pending = await invoke<string | null>(
                'take_pending_deep_link_share'
            )
            if (pending) {
                void loadShare(pending)
            }
        } catch (e) {
            await logError(`[DeepLink] Failed to check pending share: ${e}`)
        }
    })

    onUnmounted(() => {
        unlisten?.()
    })

    return {
        loading,
    }
}
