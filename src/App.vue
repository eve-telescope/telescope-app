<script setup lang="ts">
import { usePilots } from './composables/usePilots'
import { useGlobalShortcut } from './composables/useGlobalShortcut'
import { useDeepLink } from './composables/useDeepLink'
import { useUpdateChecker } from './composables/useUpdateChecker'
import { useOverlayWindow } from './composables/useOverlayWindow'
import TitleBar from './components/TitleBar.vue'
import InputPanel from './components/InputPanel.vue'
import PilotTable from './components/PilotTable.vue'
import FilterSidebar from './components/FilterSidebar.vue'
import EmptyState from './components/EmptyState.vue'
import UpdateModal from './components/UpdateModal.vue'

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

function handleGlobalPaste(text: string) {
    lookupPilots(text)
}

function handleDeepLinkPilots(pilots: string) {
    lookupPilots(pilots)
}

function handleClear() {
    clear()
    clearOverlay()
}

const { displayShortcut, updateShortcut } = useGlobalShortcut(handleGlobalPaste)

useDeepLink(handleDeepLinkPilots)

const { updateAvailable, updateInfo, dismissed, dismiss } = useUpdateChecker()
</script>

<template>
    <div class="h-screen flex flex-col overflow-hidden bg-eve-bg-0">
        <TitleBar :pilot-count="pilots.length" />

        <div class="flex-1 flex overflow-hidden">
            <!-- Left: Input Panel -->
            <InputPanel
                v-model:pilot-names="pilotNames"
                :loading="loading"
                :pilot-count="pilotCount"
                :pilots="pilots"
                :progress="progress"
                :shortcut="displayShortcut()"
                @scan="lookupPilots"
                @clear="handleClear"
                @update:shortcut="updateShortcut"
            />

            <!-- Center: Results -->
            <div class="flex-1 flex flex-col overflow-hidden">
                <PilotTable v-if="pilots.length > 0" :pilots="filteredPilots" />
                <EmptyState v-else-if="!loading" />
                <div v-else class="flex-1 flex items-center justify-center">
                    <div
                        class="w-6 h-6 border-2 border-eve-cyan border-t-transparent rounded-full animate-spin"
                    ></div>
                </div>
            </div>

            <!-- Right: Filters -->
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

        <!-- Update Modal -->
        <UpdateModal
            v-if="updateAvailable && updateInfo && !dismissed"
            :info="updateInfo"
            @dismiss="dismiss"
        />
    </div>
</template>
