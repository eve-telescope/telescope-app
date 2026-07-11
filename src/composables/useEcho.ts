import { nextTick, ref, watch } from 'vue'
import Pusher from 'pusher-js'
import { configureEcho, echo, echoIsConfigured } from '@laravel/echo-vue'
import { apiToken } from '../stores/intel'
import { API_BASE_URL } from '../utils/config'

declare global {
    interface Window {
        Pusher: typeof Pusher
    }
}

// Make Pusher available globally for Echo
window.Pusher = Pusher

export const echoReady = ref(false)

/**
 * Disconnect the current Echo socket, if any. echo-vue has no "unconfigure":
 * `echoIsConfigured()` stays true forever, so we close the underlying
 * connection ourselves. A later `configureEcho()` drops the cached instance,
 * meaning the next subscription builds a fresh connection with fresh auth.
 */
function disconnectEcho() {
    if (!echoIsConfigured()) return
    try {
        echo().disconnect()
    } catch {
        // Echo may not have instantiated a connection yet — nothing to close.
    }
}

/**
 * Configure Echo whenever the API token changes — including a rotation while
 * already authenticated, which must swap the socket's stale bearer token.
 * The actual channel subscriptions are handled by <EchoSubscriber> keyed on
 * activeNetworkId.
 */
export function useEchoConnection() {
    watch(
        apiToken,
        async (token) => {
            // Tear down on any change: logout stops here, a new/rotated
            // token reconnects below.
            echoReady.value = false
            disconnectEcho()
            if (!token) return

            // Let the echoReady=false render flush so App.vue's
            // v-if="echoReady" unmounts <EchoSubscriber>; the false→true
            // cycle forces a remount that resubscribes on the new socket.
            await nextTick()

            // The token changed again while we yielded — that newer watcher
            // invocation owns the reconnect now.
            if (token !== apiToken.value) return

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
                        Authorization: `Bearer ${token}`,
                    },
                },
            })
            echoReady.value = true
        },
        { immediate: true }
    )
}
