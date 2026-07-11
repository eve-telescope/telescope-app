<script setup lang="ts">
import { useEcho } from '@laravel/echo-vue'
import {
    onEntryDeleted,
    onScanShared,
    fetchNetworkScan,
    selectNetwork,
    activeNetworkId,
} from '../stores/intel'

const props = defineProps<{ networkId: number }>()

const channel = `intel-network.${props.networkId}`

// Events carry only IDs. Clients refetch the full data from the API.

// Refetch the selected network detail on any entry/annotation change.
useEcho<{ entry_id: number }>(
    channel,
    [
        'IntelEntryCreated',
        'IntelEntryUpdated',
        'AnnotationCreated',
        'AnnotationUpdated',
    ],
    async () => {
        if (activeNetworkId.value != null) {
            await selectNetwork(activeNetworkId.value).catch(() => {})
        }
    }
)

// Deletions can be handled locally without a refetch since we know the ID.
useEcho<{ entry_id: number }>(
    channel,
    ['IntelEntryDeleted', 'AnnotationDeleted'],
    (e) => onEntryDeleted(e.entry_id)
)

// Fetch the single scan by ID when a new one is shared.
useEcho<{ scan_id: number }>(channel, 'ScanShared', async (e) => {
    try {
        const scan = await fetchNetworkScan(props.networkId, e.scan_id)
        onScanShared(scan)
    } catch {
        // ignore
    }
})
</script>

<!-- Renderless component: only subscribes to echo events. -->
<template>
    <slot />
</template>
