import { ref, watch } from 'vue'
import Pusher from 'pusher-js'
import { configureEcho, echoIsConfigured } from '@laravel/echo-vue'
import { isAuthenticated } from '../stores/intel'
import { API_BASE_URL } from '../utils/config'
import { invoke } from '@tauri-apps/api/core'

// Make Pusher available globally for Echo
;(window as any).Pusher = Pusher

export const echoReady = ref(false)

/**
 * Configure Echo once the user is authenticated. The actual channel
 * subscriptions are handled by <EchoSubscriber> keyed on activeNetworkId.
 */
export function useEchoConnection() {
    watch(
        isAuthenticated,
        async (authed) => {
            if (!authed) {
                echoReady.value = false
                return
            }
            if (echoIsConfigured()) {
                echoReady.value = true
                return
            }
            const state = await invoke<{ api_token: string | null }>(
                'get_intel_state'
            )
            if (!state.api_token) return

            configureEcho({
                broadcaster: 'reverb',
                key:
                    import.meta.env.VITE_REVERB_APP_KEY ??
                    '04pcwy13bvcyjoio6mf6',
                wsHost:
                    import.meta.env.VITE_REVERB_HOST ?? 'ws.eve-telescope.com',
                wsPort: Number(import.meta.env.VITE_REVERB_PORT ?? 443),
                wssPort: Number(import.meta.env.VITE_REVERB_PORT ?? 443),
                forceTLS:
                    (import.meta.env.VITE_REVERB_SCHEME ?? 'https') === 'https',
                enabledTransports: ['ws', 'wss'],
                authEndpoint: `${API_BASE_URL}/broadcasting/auth`,
                auth: {
                    headers: {
                        Authorization: `Bearer ${state.api_token}`,
                    },
                },
            })
            echoReady.value = true
        },
        { immediate: true }
    )
}
