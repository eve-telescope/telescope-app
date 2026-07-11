<script setup lang="ts">
import { computed, ref, onMounted, watch } from 'vue'
import { Plus, ChevronLeft, Network, LogOut, RefreshCw } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs'
import ScanHistory from './ScanHistory.vue'
import NetworkIndexList from './network/NetworkIndexList.vue'
import NetworkEntriesTab from './network/NetworkEntriesTab.vue'
import NetworkAccessTab from './network/NetworkAccessTab.vue'
import AnnotationDialog from './network/AnnotationDialog.vue'
import NetworkAccessDialog from './network/NetworkAccessDialog.vue'
import {
    isAuthenticated,
    networks,
    activeNetworkId,
    selectedNetwork,
    selectedNetworkAnnotations,
    fetchNetworks,
    createNetwork,
    deleteNetwork,
    selectNetwork,
    setActiveNetworkId,
    clearSelectedNetwork,
    createAnnotation,
    updateAnnotation,
    removeEntry,
    addAccess,
    removeAccess,
} from '../stores/intel'
import type { IntelAnnotation } from '../types'
import {
    isAnnotationFormEmpty,
    resolveAnnotationSaveAction,
    type AccessSavePayload,
    type AnnotationSavePayload,
} from '../utils/network'

const props = withDefaults(
    defineProps<{
        embedded?: boolean
    }>(),
    {
        embedded: false,
    }
)

const emit = defineEmits<{
    loadScan: [rawText: string]
}>()

// ---------------------------------------------------------------------------
// View state
// ---------------------------------------------------------------------------

const view = ref<'list' | 'detail'>('list')
const detailTab = ref<'annotations' | 'access' | 'scans'>('annotations')
const loading = ref(false)
const error = ref<string | null>(null)

const currentNetwork = computed(() => selectedNetwork.value ?? null)

// ---------------------------------------------------------------------------
// Network CRUD
// ---------------------------------------------------------------------------

const newNetworkName = ref('')

async function handleCreateNetwork() {
    if (!newNetworkName.value.trim()) return
    loading.value = true
    error.value = null
    try {
        await createNetwork(newNetworkName.value.trim())
        newNetworkName.value = ''
    } catch (e) {
        error.value = String(e)
    } finally {
        loading.value = false
    }
}

async function handleSelectNetwork(networkId: number) {
    loading.value = true
    error.value = null
    try {
        await selectNetwork(networkId)
        detailTab.value = 'annotations'
        view.value = 'detail'
    } catch (e) {
        error.value = String(e)
    } finally {
        loading.value = false
    }
}

const refreshing = ref(false)

async function handleRefreshNetwork() {
    const net = currentNetwork.value
    if (!net) return
    refreshing.value = true
    try {
        await selectNetwork(net.id)
    } catch (e) {
        error.value = String(e)
    } finally {
        refreshing.value = false
    }
}

async function handleConnectNetwork() {
    const net = selectedNetwork.value
    if (!net) return
    loading.value = true
    error.value = null
    try {
        await setActiveNetworkId(net.id)
    } catch (e) {
        error.value = String(e)
    } finally {
        loading.value = false
    }
}

const togglingId = ref<number | null>(null)

async function handleToggleConnect(networkId: number) {
    togglingId.value = networkId
    error.value = null
    try {
        await setActiveNetworkId(
            activeNetworkId.value === networkId ? null : networkId
        )
    } catch (e) {
        error.value = String(e)
    } finally {
        togglingId.value = null
    }
}

async function handleDisconnectNetwork() {
    loading.value = true
    error.value = null
    try {
        await setActiveNetworkId(null)
        view.value = 'list'
    } catch (e) {
        error.value = String(e)
    } finally {
        loading.value = false
    }
}

async function handleDeleteNetwork(id: number) {
    loading.value = true
    try {
        await deleteNetwork(id)
        if (selectedNetwork.value?.id === id) {
            view.value = 'list'
        }
    } catch (e) {
        error.value = String(e)
    } finally {
        loading.value = false
    }
}

function handleBack() {
    clearSelectedNetwork()
    view.value = 'list'
}

// ---------------------------------------------------------------------------
// Annotation dialog
// ---------------------------------------------------------------------------

const showAnnotationDialog = ref(false)
const editingAnnotation = ref<IntelAnnotation | null>(null)

function openAnnotationDialog() {
    editingAnnotation.value = null
    showAnnotationDialog.value = true
}

function startEditAnnotation(annotation: IntelAnnotation) {
    editingAnnotation.value = annotation
    showAnnotationDialog.value = true
}

async function handleSaveAnnotation(payload: AnnotationSavePayload) {
    const net = selectedNetwork.value
    if (!net) return
    const action = resolveAnnotationSaveAction(
        payload.editingId,
        isAnnotationFormEmpty(payload.tags, payload.note)
    )
    loading.value = true
    error.value = null
    try {
        if (action === 'delete') {
            await removeEntry(net.id, payload.editingId!)
        } else if (action === 'update') {
            await updateAnnotation(
                net.id,
                payload.editingId!,
                payload.entity.type,
                parseInt(payload.entity.id),
                payload.entity.name,
                payload.tags,
                payload.note
            )
        } else if (action === 'create') {
            await createAnnotation(
                net.id,
                payload.entity.type,
                parseInt(payload.entity.id),
                payload.entity.name,
                payload.tags,
                payload.note
            )
        }
        editingAnnotation.value = null
        showAnnotationDialog.value = false
    } catch (e) {
        error.value = String(e)
    } finally {
        loading.value = false
    }
}

async function handleDeleteEntry(entryId: number) {
    const net = selectedNetwork.value
    if (!net) return
    loading.value = true
    try {
        await removeEntry(net.id, entryId)
    } catch (e) {
        error.value = String(e)
    } finally {
        loading.value = false
    }
}

// ---------------------------------------------------------------------------
// Access dialog
// ---------------------------------------------------------------------------

const showAccessDialog = ref(false)

function openAccessDialog() {
    showAccessDialog.value = true
}

async function handleAddAccess(payload: AccessSavePayload) {
    const net = selectedNetwork.value
    if (!net) return
    loading.value = true
    error.value = null
    try {
        await addAccess(
            net.id,
            payload.entity.type,
            parseInt(payload.entity.id),
            payload.entity.name,
            payload.permission
        )
        showAccessDialog.value = false
    } catch (e) {
        error.value = String(e)
    } finally {
        loading.value = false
    }
}

async function handleDeleteAccess(accessId: number) {
    const net = selectedNetwork.value
    if (!net) return
    loading.value = true
    try {
        await removeAccess(net.id, accessId)
    } catch (e) {
        error.value = String(e)
    } finally {
        loading.value = false
    }
}

onMounted(() => {
    fetchNetworks()
})

// Keep the active network's data loaded so the index can show live status and
// the detail view is ready when opened — but don't hijack the view: the list is
// the hub where connections are managed.
watch(
    [isAuthenticated, networks, activeNetworkId],
    async ([authed, currentNetworks, currentActiveId]) => {
        if (!props.embedded || !authed) return
        if (currentActiveId == null) return
        const exists = currentNetworks.some((n) => n.id === currentActiveId)
        if (exists && selectedNetwork.value?.id !== currentActiveId) {
            await selectNetwork(currentActiveId)
        }
    },
    { immediate: true }
)
</script>

<template>
    <div
        :class="
            props.embedded
                ? 'h-full flex flex-col bg-eve-bg-0'
                : 'h-screen flex flex-col bg-eve-bg-0'
        "
    >
        <div class="flex-1 flex flex-col overflow-hidden">
            <!-- Header (standalone window only) -->
            <div
                v-if="!props.embedded"
                data-tauri-drag-region
                class="flex items-center gap-2 px-4 py-3 border-b border-eve-border shrink-0 bg-eve-bg-1/80 backdrop-blur-sm"
            >
                <button
                    v-if="view === 'detail'"
                    class="p-1 rounded text-eve-text-3 hover:text-eve-text-1 hover:bg-eve-bg-hover transition-colors"
                    @click="handleBack"
                >
                    <ChevronLeft class="w-4 h-4" />
                </button>
                <h2 class="text-sm font-bold tracking-wider text-eve-text-1">
                    <template v-if="view === 'list'">INTEL NETWORKS</template>
                    <template v-else>{{ selectedNetwork?.name }}</template>
                </h2>
            </div>

            <!-- Error -->
            <div
                v-if="error"
                class="px-4 py-2 bg-eve-red/10 border-b border-eve-red/20 text-eve-red text-xs flex items-center justify-between"
            >
                <span>{{ error }}</span>
                <button
                    class="text-eve-red/60 hover:text-eve-red"
                    @click="error = null"
                >
                    &times;
                </button>
            </div>

            <!-- Content -->
            <div class="flex-1 overflow-y-auto">
                <!-- Not authenticated -->
                <template v-if="!isAuthenticated">
                    <div
                        class="flex flex-col items-center justify-center h-full px-6 text-center"
                    >
                        <div
                            class="w-10 h-10 rounded-full bg-eve-bg-2 flex items-center justify-center mb-3"
                        >
                            <Network class="w-5 h-5 text-eve-text-3" />
                        </div>
                        <h3 class="text-sm font-semibold text-eve-text-1 mb-1">
                            Intel Networks
                        </h3>
                        <p class="text-xs text-eve-text-3 max-w-xs">
                            Sign in via the Settings tab to access your intel
                            networks.
                        </p>
                    </div>
                </template>

                <!-- ==================== NETWORK LIST ==================== -->
                <template v-else-if="view === 'list'">
                    <NetworkIndexList
                        v-model="newNetworkName"
                        :networks="networks"
                        :active-network-id="activeNetworkId"
                        :loading="loading"
                        :toggling-id="togglingId"
                        @create="handleCreateNetwork"
                        @select="handleSelectNetwork"
                        @toggle-connect="handleToggleConnect"
                        @delete="handleDeleteNetwork"
                    />
                </template>

                <!-- ==================== NETWORK DETAIL ==================== -->
                <template v-else-if="view === 'detail' && currentNetwork">
                    <div class="flex flex-col h-full">
                        <!-- Network header -->
                        <div
                            class="flex items-center gap-3 px-4 py-3 border-b border-eve-border bg-eve-bg-1/50"
                        >
                            <div
                                class="w-8 h-8 rounded bg-eve-cyan/10 flex items-center justify-center shrink-0"
                            >
                                <Network class="w-4 h-4 text-eve-cyan" />
                            </div>
                            <div class="flex-1 min-w-0">
                                <div
                                    class="text-xs font-semibold text-eve-text-1 truncate"
                                >
                                    {{ currentNetwork.name }}
                                </div>
                                <div class="flex items-center gap-3 mt-0.5">
                                    <span class="text-[10px] text-eve-text-3"
                                        >{{
                                            selectedNetworkAnnotations.length
                                        }}
                                        annotations</span
                                    >
                                    <span class="text-[10px] text-eve-text-3"
                                        >{{
                                            currentNetwork.accesses.length
                                        }}
                                        members</span
                                    >
                                </div>
                            </div>
                            <Button
                                variant="ghost"
                                size="sm"
                                class="text-eve-text-3 hover:text-eve-text-1 h-7 w-7 p-0"
                                :disabled="refreshing"
                                title="Refresh"
                                @click="handleRefreshNetwork"
                            >
                                <RefreshCw
                                    class="w-3.5 h-3.5"
                                    :class="refreshing ? 'animate-spin' : ''"
                                />
                            </Button>
                            <Button
                                v-if="activeNetworkId === currentNetwork.id"
                                variant="ghost"
                                size="sm"
                                class="text-eve-text-3 hover:text-eve-text-1 h-7 px-2 text-[10px]"
                                @click="handleDisconnectNetwork"
                            >
                                <LogOut class="w-3.5 h-3.5 mr-1" />
                                Disconnect
                            </Button>
                            <Button
                                v-else
                                size="sm"
                                class="h-7 px-3 text-[10px]"
                                @click="handleConnectNetwork"
                            >
                                Connect
                            </Button>
                        </div>

                        <!-- Tabs -->
                        <Tabs
                            v-model:model-value="detailTab"
                            class="flex-1 flex flex-col min-h-0"
                        >
                            <div
                                class="flex items-center justify-between px-4 py-2 border-b border-eve-border"
                            >
                                <TabsList class="h-7">
                                    <TabsTrigger
                                        value="annotations"
                                        class="text-[10px] h-6 px-3"
                                        >Annotations</TabsTrigger
                                    >
                                    <TabsTrigger
                                        value="scans"
                                        class="text-[10px] h-6 px-3"
                                        >Scans</TabsTrigger
                                    >
                                    <TabsTrigger
                                        value="access"
                                        class="text-[10px] h-6 px-3"
                                        >Access</TabsTrigger
                                    >
                                </TabsList>

                                <Button
                                    v-if="detailTab === 'annotations'"
                                    variant="ghost"
                                    size="sm"
                                    class="h-7 px-2 text-[10px] text-eve-cyan hover:text-eve-cyan hover:bg-eve-cyan/10"
                                    @click="openAnnotationDialog"
                                >
                                    <Plus class="w-3 h-3 mr-1" />
                                    Add
                                </Button>

                                <Button
                                    v-else-if="detailTab === 'access'"
                                    variant="ghost"
                                    size="sm"
                                    class="h-7 px-2 text-[10px] text-eve-cyan hover:text-eve-cyan hover:bg-eve-cyan/10"
                                    @click="openAccessDialog"
                                >
                                    <Plus class="w-3 h-3 mr-1" />
                                    Grant Access
                                </Button>
                            </div>

                            <!-- Annotations tab -->
                            <TabsContent
                                value="annotations"
                                class="mt-0 flex-1 overflow-y-auto"
                            >
                                <NetworkEntriesTab
                                    :annotations="selectedNetworkAnnotations"
                                    @add="openAnnotationDialog"
                                    @edit="startEditAnnotation"
                                    @delete="handleDeleteEntry"
                                />
                            </TabsContent>

                            <!-- Access tab -->
                            <TabsContent
                                value="access"
                                class="mt-0 flex-1 overflow-y-auto"
                            >
                                <NetworkAccessTab
                                    :accesses="currentNetwork.accesses"
                                    @delete="handleDeleteAccess"
                                />
                            </TabsContent>

                            <!-- Scans tab -->
                            <TabsContent
                                value="scans"
                                class="mt-0 flex-1 overflow-y-auto"
                            >
                                <ScanHistory
                                    @load-scan="emit('loadScan', $event)"
                                />
                            </TabsContent>
                        </Tabs>
                    </div>
                </template>
            </div>
        </div>

        <!-- ==================== ANNOTATION DIALOG ==================== -->
        <AnnotationDialog
            v-model:open="showAnnotationDialog"
            :loading="loading"
            :editing="editingAnnotation"
            @save="handleSaveAnnotation"
        />

        <!-- ==================== ACCESS DIALOG ==================== -->
        <NetworkAccessDialog
            v-model:open="showAccessDialog"
            :loading="loading"
            @save="handleAddAccess"
        />
    </div>
</template>

<style>
[data-tauri-drag-region] {
    -webkit-user-select: none;
    user-select: none;
    -webkit-app-region: drag;
    app-region: drag;
}

[data-tauri-drag-region] button {
    -webkit-app-region: no-drag;
    app-region: no-drag;
}
</style>
