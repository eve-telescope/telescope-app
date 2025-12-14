import { ref, onMounted, onUnmounted } from 'vue'
import { onOpenUrl, getCurrent } from '@tauri-apps/plugin-deep-link'
import { parseDeepLinkUrl } from '../utils/share'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { info, error as logError, warn } from '@tauri-apps/plugin-log'

const API_BASE_URL = 'https://eve-telescope.com'

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

export function useDeepLink(onPilotsReceived: (pilots: string) => void) {
    const lastUrl = ref<string | null>(null)
    const loading = ref(false)
    let unlisten: UnlistenFn | null = null

    async function handleUrl(url: string) {
        await info(`[DeepLink] Received URL: ${url}`)
        lastUrl.value = url
        const code = parseDeepLinkUrl(url)
        await info(`[DeepLink] Parsed code: ${code}`)

        if (!code) {
            await warn('[DeepLink] Failed to parse code from URL')
            return
        }

        loading.value = true
        try {
            await info('[DeepLink] Fetching share data...')
            const share = await fetchShare(code)
            await info(`[DeepLink] Share data: ${JSON.stringify(share)}`)
            if (share?.pilots) {
                await info(
                    `[DeepLink] Calling onPilotsReceived with ${share.pilots.length} pilots`
                )
                onPilotsReceived(share.pilots.join('\n'))
            }
        } catch (e) {
            await logError(`[DeepLink] Error fetching share: ${e}`)
        } finally {
            loading.value = false
        }
    }

    async function checkStartupUrl() {
        try {
            await info('[DeepLink] Checking startup URL...')
            const urls = await getCurrent()
            await info(`[DeepLink] Startup URLs: ${JSON.stringify(urls)}`)
            if (urls && urls.length > 0) {
                handleUrl(urls[0])
            }
        } catch (e) {
            await logError(`[DeepLink] Failed to get startup deep link: ${e}`)
        }
    }

    async function setupListener() {
        unlisten = await onOpenUrl((urls) => {
            for (const url of urls) {
                handleUrl(url)
                break
            }
        })
    }

    onMounted(() => {
        checkStartupUrl()
        setupListener()
    })

    onUnmounted(() => {
        unlisten?.()
    })

    return {
        lastUrl,
        loading,
    }
}
