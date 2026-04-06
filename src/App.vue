<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { usePilots } from './composables/usePilots'
import { useDscan } from './composables/useDscan'
import { useGlobalShortcut } from './composables/useGlobalShortcut'
import { useDeepLink } from './composables/useDeepLink'
import { useUpdateChecker } from './composables/useUpdateChecker'
import { useOverlayWindow } from './composables/useOverlayWindow'
import { useEchoConnection, echoReady } from './composables/useEcho'
import EchoSubscriber from './components/EchoSubscriber.vue'
import { Tabs, TabsContent } from '@/components/ui/tabs'
import TitleBar from './components/TitleBar.vue'
import InputPanel from './components/InputPanel.vue'
import PilotTable from './components/PilotTable.vue'
import FilterSidebar from './components/FilterSidebar.vue'
import EmptyState from './components/EmptyState.vue'
import UpdateModal from './components/UpdateModal.vue'
import NetworkManager from './components/NetworkManager.vue'
import SettingsPanel from './components/SettingsPanel.vue'
import DscanPanel from './components/DscanPanel.vue'
import { detectScanInputKind } from './utils/scanInput'
import { isAuthenticated, activeNetworkId, shareScan } from './stores/intel'

const {
    pilotNames,
    pilots,
    filteredPilots,
    loading,
    progress,
    pilotCount,
    selectedCorps,
    selectedAlliances,
    selectedTags,
    lookupPilots,
    toggleCorp,
    toggleAlliance,
    toggleTag,
    clearFilters,
    clear,
} = usePilots()

const { clearOverlay } = useOverlayWindow()
const {
    rawInput: dscanRawInput,
    result: dscanResult,
    sdeStatus,
    loading: dscanLoading,
    syncing: dscanSyncing,
    error: dscanError,
    ensureSdeIndex,
    parse: parseDscan,
    clear: clearDscan,
} = useDscan()

async function loadScan(text: string) {
    const kind = detectScanInputKind(text)

    if (kind === 'dscan') {
        activeTab.value = 'dscan'
        await parseDscan(text)
    } else {
        activeTab.value = 'local'
        await lookupPilots(text)
    }
}

async function handleScanInput(text: string) {
    await loadScan(text)

    // Auto-share with active network — the Echo broadcast will update the history
    if (isAuthenticated.value && activeNetworkId.value != null) {
        shareScan(activeNetworkId.value, detectScanInputKind(text), text).catch(() => {})
    }
}

function handleGlobalPaste(text: string) {
    void handleScanInput(text)
}

function handleDeepLinkPilots(pilots: string) {
    void handleScanInput(pilots)
}

function handleClear() {
    clear()
    clearDscan()
    clearOverlay()
}

const { displayShortcut, updateShortcut } = useGlobalShortcut(handleGlobalPaste)

useDeepLink(handleDeepLinkPilots)

const { updateAvailable, updateInfo, dismissed, dismiss } = useUpdateChecker()

const activeTab = ref<'local' | 'dscan' | 'networks' | 'settings'>('local')
const sharedScanLoading = computed(
    () => loading.value || dscanLoading.value || dscanSyncing.value
)

onMounted(() => {
    void ensureSdeIndex()
})

useEchoConnection()
</script>

<template>
    <Tabs
        v-model:model-value="activeTab"
        class="h-screen flex flex-col overflow-hidden bg-eve-bg-0 gap-0"
    >
        <TitleBar
            :pilot-count="pilots.length"
            :active-tab="activeTab"
            @update:active-tab="activeTab = $event as typeof activeTab"
        />

        <div class="flex-1 flex overflow-hidden">
            <InputPanel
                v-model:pilot-names="pilotNames"
                :loading="sharedScanLoading"
                :pilot-count="pilotCount"
                :pilots="pilots"
                :progress="progress"
                @scan="handleScanInput(pilotNames)"
                @clear="handleClear"
                @load-scan="loadScan"
            />

            <TabsContent value="local" class="mt-0 flex-1 overflow-hidden">
                <div class="flex h-full overflow-hidden">
                    <div class="flex-1 flex flex-col overflow-hidden">
                        <PilotTable
                            v-if="pilots.length > 0"
                            :pilots="filteredPilots"
                        />
                        <EmptyState v-else-if="!loading" />
                        <div
                            v-else
                            class="flex-1 flex items-center justify-center"
                        >
                            <div
                                class="w-6 h-6 border-2 border-eve-cyan border-t-transparent rounded-full animate-spin"
                            ></div>
                        </div>
                    </div>

                    <FilterSidebar
                        v-if="pilots.length > 0"
                        :pilots="pilots"
                        :selected-corps="selectedCorps"
                        :selected-alliances="selectedAlliances"
                        :selected-tags="selectedTags"
                        @toggle-corp="toggleCorp"
                        @toggle-alliance="toggleAlliance"
                        @toggle-tag="toggleTag"
                        @clear-filters="clearFilters"
                    />
                </div>
            </TabsContent>

            <TabsContent value="dscan" class="mt-0 flex-1 overflow-hidden">
                <DscanPanel
                    :raw-input="dscanRawInput"
                    :result="dscanResult"
                    :loading="dscanLoading"
                    :error="dscanError"
                />
            </TabsContent>

            <TabsContent value="networks" class="mt-0 flex-1 overflow-hidden">
                <NetworkManager embedded @load-scan="loadScan" />
            </TabsContent>

            <TabsContent value="settings" class="mt-0 flex-1 overflow-hidden">
                <SettingsPanel
                    :shortcut="displayShortcut()"
                    :sde-status="sdeStatus"
                    :sde-syncing="dscanSyncing"
                    @update-shortcut="updateShortcut"
                    @refresh-sde="ensureSdeIndex"
                />
            </TabsContent>
        </div>

        <!-- Echo subscriber, re-mounts when active network changes -->
        <EchoSubscriber
            v-if="echoReady && activeNetworkId != null"
            :key="activeNetworkId"
            :network-id="activeNetworkId"
        />

        <!-- Update Modal -->
        <UpdateModal
            v-if="updateAvailable && updateInfo && !dismissed"
            :info="updateInfo"
            @dismiss="dismiss"
        />
    </Tabs>
</template>
