import Echo from 'laravel-echo'
import Pusher from 'pusher-js'
import { watch, ref } from 'vue'
import {
    isAuthenticated,
    activeNetworkId,
    onEntryCreated,
    onEntryUpdated,
    onEntryDeleted,
    onScanShared,
} from '../stores/intel'
import { info } from '@tauri-apps/plugin-log'
import type { IntelEntry, NetworkScan } from '../types'
import { API_BASE_URL } from '../utils/config'
import { invoke } from '@tauri-apps/api/core'

// Make Pusher available globally for Echo
;(window as any).Pusher = Pusher

let echo: Echo<'reverb'> | null = null
const connected = ref(false)

type EntryEventPayload = { entry: IntelEntry }
type EntryDeletePayload = { entry_id: number }

function registerEntryListeners(
    channel: ReturnType<Echo<'reverb'>['private']>,
    networkId: number
) {
    channel
        .listen('IntelEntryCreated', (e: EntryEventPayload) => {
            info(`[Echo] Entry created in network ${networkId}`)
            onEntryCreated(e.entry)
        })
        .listen('IntelEntryUpdated', (e: EntryEventPayload) => {
            info(`[Echo] Entry updated in network ${networkId}`)
            onEntryUpdated(e.entry)
        })
        .listen('IntelEntryDeleted', (e: EntryDeletePayload) => {
            info(`[Echo] Entry deleted in network ${networkId}`)
            onEntryDeleted(e.entry_id)
        })
        .listen('AnnotationCreated', (e: EntryEventPayload) => {
            info(`[Echo] Annotation created in network ${networkId}`)
            onEntryCreated(e.entry)
        })
        .listen('AnnotationUpdated', (e: EntryEventPayload) => {
            info(`[Echo] Annotation updated in network ${networkId}`)
            onEntryUpdated(e.entry)
        })
        .listen('AnnotationDeleted', (e: EntryDeletePayload) => {
            info(`[Echo] Annotation deleted in network ${networkId}`)
            onEntryDeleted(e.entry_id)
        })
        .listen('ScanShared', (e: { scan: NetworkScan }) => {
            info(`[Echo] Scan shared in network ${networkId}`)
            onScanShared(e.scan)
        })
}

export function useEchoConnection() {
    async function connect() {
        if (echo) return

        // Get token from Rust state
        const state = await invoke<{ api_token: string | null }>(
            'get_intel_state'
        )
        const token = state.api_token
        if (!token) return

        echo = new Echo({
            broadcaster: 'reverb',
            key: import.meta.env.VITE_REVERB_APP_KEY ?? 'laravel-herd',
            wsHost: import.meta.env.VITE_REVERB_HOST ?? 'reverb.herd.test',
            wsPort: import.meta.env.VITE_REVERB_PORT ?? 443,
            wssPort: import.meta.env.VITE_REVERB_PORT ?? 443,
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

        connected.value = true
        info('[Echo] Connected to Reverb')
    }

    function disconnect() {
        if (echo) {
            echo.disconnect()
            echo = null
            connected.value = false
            info('[Echo] Disconnected')
        }
    }

    function subscribeToNetwork(networkId: number) {
        if (!echo) return

        const channel = echo.private(`intel-network.${networkId}`)
        registerEntryListeners(channel, networkId)

        info(`[Echo] Subscribed to intel-network.${networkId}`)
    }

    function unsubscribeFromNetwork(networkId: number) {
        if (!echo) return
        echo.leave(`intel-network.${networkId}`)
        info(`[Echo] Unsubscribed from intel-network.${networkId}`)
    }

    // Auto-connect/disconnect when auth changes
    watch(
        isAuthenticated,
        (authed) => {
            if (authed) {
                disconnect()
                connect()
            } else {
                disconnect()
            }
        },
        { immediate: true }
    )

    // Subscribe/unsubscribe when active network changes
    watch(
        activeNetworkId,
        (newId, oldId) => {
            if (oldId != null && oldId !== newId) {
                unsubscribeFromNetwork(oldId)
            }

            if (newId != null && newId !== oldId) {
                subscribeToNetwork(newId)
            }
        },
        { immediate: true }
    )

    return {
        connected,
        connect,
        disconnect,
        subscribeToNetwork,
        unsubscribeFromNetwork,
    }
}
