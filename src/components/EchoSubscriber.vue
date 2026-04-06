<script setup lang="ts">
import { useEcho } from '@laravel/echo-vue'
import {
    onEntryCreated,
    onEntryUpdated,
    onEntryDeleted,
    onScanShared,
} from '../stores/intel'
import type { IntelEntry, NetworkScan } from '../types'

const props = defineProps<{ networkId: number }>()

const channel = `intel-network.${props.networkId}`

useEcho<{ entry: IntelEntry }>(
    channel,
    ['IntelEntryCreated', 'AnnotationCreated'],
    (e) => onEntryCreated(e.entry)
)

useEcho<{ entry: IntelEntry }>(
    channel,
    ['IntelEntryUpdated', 'AnnotationUpdated'],
    (e) => onEntryUpdated(e.entry)
)

useEcho<{ entry_id: number }>(
    channel,
    ['IntelEntryDeleted', 'AnnotationDeleted'],
    (e) => onEntryDeleted(e.entry_id)
)

useEcho<{ scan: NetworkScan }>(
    channel,
    'ScanShared',
    (e) => onScanShared(e.scan)
)
</script>

<template>
    <!-- No UI, this component only subscribes to echo events -->
</template>
