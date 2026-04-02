<script setup lang="ts">
import { computed, ref, onMounted, watch } from 'vue'
import {
    Plus,
    Trash2,
    ChevronLeft,
    Shield,
    Pencil,
    Network,
    LogOut,
    Zap,
} from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs'
import { Badge } from '@/components/ui/badge'
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'
import EntityCombobox from './EntityCombobox.vue'
import ScanHistory from './ScanHistory.vue'
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
import type { EntityType, PermissionLevel } from '../types'
import {
    PRESET_ANNOTATION_TAGS,
    getAnnotationColor,
    normalizeAnnotationTag,
    parseAnnotationTags,
} from '../utils/annotations'

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
const editingAnnotationId = ref<number | null>(null)

const annotationEntity = ref<{
    id: string
    name: string
    type: EntityType | null
}>({
    id: '',
    name: '',
    type: null,
})

const entryForm = ref<{
    tags: string
    notes: string
}>({
    tags: '',
    notes: '',
})

function resetEntryForm() {
    editingAnnotationId.value = null
    annotationEntity.value = { id: '', name: '', type: null }
    entryForm.value = {
        tags: '',
        notes: '',
    }
}

function openAnnotationDialog() {
    resetEntryForm()
    showAnnotationDialog.value = true
}

function startEditAnnotation(annotation: {
    id: number
    targetType: EntityType
    targetId: number
    targetName: string
    color: string | null
    tags: string[]
    note: string | null
}) {
    editingAnnotationId.value = annotation.id
    annotationEntity.value = {
        id: String(annotation.targetId),
        name: annotation.targetName,
        type: annotation.targetType,
    }
    entryForm.value = {
        tags: annotation.tags.join(' | '),
        notes: annotation.note ?? '',
    }
    showAnnotationDialog.value = true
}

function togglePresetTag(tag: string) {
    const tags = parseAnnotationTags(entryForm.value.tags)
    const normalized = normalizeAnnotationTag(tag)
    const idx = tags.indexOf(normalized)
    if (idx >= 0) {
        tags.splice(idx, 1)
    } else {
        tags.push(normalized)
    }
    entryForm.value.tags = tags.join(' | ')
}

async function handleAddEntry() {
    const net = selectedNetwork.value
    const entity = annotationEntity.value
    const tags = parseAnnotationTags(entryForm.value.tags)
    const note = entryForm.value.notes || null
    const isEmpty = tags.length === 0 && !note
    if (!net || !entity.id || !entity.type) return
    loading.value = true
    error.value = null
    try {
        if (isEmpty && editingAnnotationId.value != null) {
            await removeEntry(net.id, editingAnnotationId.value)
        } else if (!isEmpty && editingAnnotationId.value != null) {
            await updateAnnotation(
                net.id,
                editingAnnotationId.value,
                entity.type,
                parseInt(entity.id),
                entity.name,
                tags,
                note
            )
        } else if (!isEmpty) {
            await createAnnotation(
                net.id,
                entity.type,
                parseInt(entity.id),
                entity.name,
                tags,
                note
            )
        }
        resetEntryForm()
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
const accessPermission = ref<PermissionLevel>('viewer')
const accessEntity = ref<{ id: string; name: string; type: EntityType | null }>(
    {
        id: '',
        name: '',
        type: null,
    }
)

function openAccessDialog() {
    accessEntity.value = { id: '', name: '', type: null }
    accessPermission.value = 'viewer'
    showAccessDialog.value = true
}

async function handleAddAccess() {
    const net = selectedNetwork.value
    const entity = accessEntity.value
    if (!net || !entity.id || !entity.type) return
    loading.value = true
    error.value = null
    try {
        await addAccess(
            net.id,
            entity.type,
            parseInt(entity.id),
            entity.name,
            accessPermission.value
        )
        accessEntity.value = { id: '', name: '', type: null }
        accessPermission.value = 'viewer'
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

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function getPortraitUrl(type: string, id: number, size = 32): string {
    if (type === 'character' || type.includes('User')) {
        return `https://images.evetech.net/characters/${id}/portrait?size=${size}`
    }
    if (type === 'corporation' || type.includes('Corporation')) {
        return `https://images.evetech.net/corporations/${id}/logo?size=${size}`
    }
    if (type === 'alliance' || type.includes('Alliance')) {
        return `https://images.evetech.net/alliances/${id}/logo?size=${size}`
    }
    return ''
}

onMounted(() => {
    fetchNetworks()
})

watch(
    [isAuthenticated, networks, activeNetworkId],
    async ([authed, currentNetworks, currentActiveId]) => {
        if (!props.embedded || !authed) return
        if (currentNetworks.length === 0) {
            view.value = 'list'
            return
        }
        if (currentActiveId != null) {
            const exists = currentNetworks.some((n) => n.id === currentActiveId)
            if (exists) {
                if (selectedNetwork.value?.id !== currentActiveId) {
                    await selectNetwork(currentActiveId)
                }
                view.value = 'detail'
                return
            }
        }
        view.value = 'list'
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
                    class="p-1 rounded text-eve-text-3 hover:text-eve-text-1 hover:bg-white/5 transition-colors"
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
                    <div class="p-4 space-y-3">
                        <div class="flex gap-2">
                            <Input
                                v-model="newNetworkName"
                                type="text"
                                placeholder="New network name..."
                                class="flex-1"
                                @keydown.enter="handleCreateNetwork"
                            />
                            <Button
                                size="sm"
                                :disabled="!newNetworkName.trim() || loading"
                                @click="handleCreateNetwork"
                            >
                                <Plus class="w-3.5 h-3.5" />
                                Create
                            </Button>
                        </div>

                        <div
                            v-if="networks.length === 0"
                            class="flex flex-col items-center py-12 text-center"
                        >
                            <div
                                class="w-10 h-10 rounded-full bg-eve-bg-2 flex items-center justify-center mb-3"
                            >
                                <Network class="w-5 h-5 text-eve-text-3" />
                            </div>
                            <p class="text-xs text-eve-text-3">
                                No networks yet. Create one to get started.
                            </p>
                        </div>

                        <div
                            v-for="network in networks"
                            :key="network.id"
                            class="group flex items-center gap-3 p-3 rounded-lg border transition-colors cursor-pointer"
                            :class="
                                activeNetworkId === network.id
                                    ? 'border-eve-cyan/30 bg-eve-cyan/5'
                                    : 'border-eve-border bg-eve-bg-1 hover:border-eve-text-3/30 hover:bg-eve-bg-2'
                            "
                            @click="handleSelectNetwork(network.id)"
                        >
                            <div
                                class="w-8 h-8 rounded bg-eve-bg-2 flex items-center justify-center shrink-0"
                            >
                                <Network
                                    class="w-4 h-4"
                                    :class="
                                        activeNetworkId === network.id
                                            ? 'text-eve-cyan'
                                            : 'text-eve-text-3'
                                    "
                                />
                            </div>
                            <div class="flex-1 min-w-0">
                                <div
                                    class="text-xs font-medium text-eve-text-1 truncate"
                                >
                                    {{ network.name }}
                                </div>
                                <div class="text-[10px] text-eve-text-3 mt-0.5">
                                    {{ network.entries_count ?? 0 }} annotations
                                </div>
                            </div>
                            <Badge
                                v-if="activeNetworkId === network.id"
                                variant="outline"
                                class="border-eve-cyan/40 text-eve-cyan text-[9px] shrink-0"
                                >Connected</Badge
                            >
                            <button
                                class="p-1.5 rounded text-eve-text-3 opacity-0 group-hover:opacity-100 hover:text-eve-red hover:bg-eve-red/10 transition-all shrink-0"
                                title="Delete network"
                                @click.stop="handleDeleteNetwork(network.id)"
                            >
                                <Trash2 class="w-3 h-3" />
                            </button>
                        </div>
                    </div>
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
                                <div
                                    v-if="
                                        selectedNetworkAnnotations.length === 0
                                    "
                                    class="flex flex-col items-center justify-center py-12 text-center"
                                >
                                    <div
                                        class="w-10 h-10 rounded-full bg-eve-bg-2 flex items-center justify-center mb-3"
                                    >
                                        <Zap class="w-5 h-5 text-eve-text-3" />
                                    </div>
                                    <p class="text-xs text-eve-text-3">
                                        No annotations yet
                                    </p>
                                    <Button
                                        variant="ghost"
                                        size="sm"
                                        class="mt-2 text-[10px] text-eve-cyan hover:text-eve-cyan"
                                        @click="openAnnotationDialog"
                                    >
                                        <Plus class="w-3 h-3 mr-1" />
                                        Add your first annotation
                                    </Button>
                                </div>

                                <div
                                    v-else
                                    class="divide-y divide-eve-border/50"
                                >
                                    <div
                                        v-for="entry in selectedNetworkAnnotations"
                                        :key="entry.id"
                                        class="group flex items-start gap-3 px-4 py-2.5 hover:bg-eve-bg-1/50 transition-colors"
                                    >
                                        <div class="flex-1 min-w-0">
                                            <div
                                                class="flex items-center gap-2"
                                            >
                                                <span
                                                    class="text-xs font-medium text-eve-text-1 truncate"
                                                    >{{
                                                        entry.targetName
                                                    }}</span
                                                >
                                                <span
                                                    class="text-[9px] uppercase text-eve-text-3"
                                                    >{{
                                                        entry.targetType
                                                    }}</span
                                                >
                                            </div>
                                            <div
                                                class="flex flex-wrap gap-1 mt-1"
                                            >
                                                <Badge
                                                    v-for="tag in entry.tags"
                                                    :key="`${entry.id}:${tag}`"
                                                    variant="secondary"
                                                    class="text-[9px] h-4 px-1.5"
                                                    :style="{
                                                        backgroundColor:
                                                            (entry.color ||
                                                                '#94A3B8') +
                                                            '22',
                                                        color:
                                                            entry.color ||
                                                            '#CBD5E1',
                                                    }"
                                                    >{{ tag }}</Badge
                                                >
                                            </div>
                                            <p
                                                v-if="entry.note"
                                                class="text-[10px] text-eve-text-3 mt-1 truncate"
                                            >
                                                {{ entry.note }}
                                            </p>
                                        </div>
                                        <div
                                            class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity shrink-0 pt-0.5"
                                        >
                                            <button
                                                class="p-1 rounded text-eve-text-3 hover:text-eve-cyan hover:bg-eve-cyan/10 transition-colors"
                                                @click="
                                                    startEditAnnotation(entry)
                                                "
                                            >
                                                <Pencil class="w-3 h-3" />
                                            </button>
                                            <button
                                                class="p-1 rounded text-eve-text-3 hover:text-eve-red hover:bg-eve-red/10 transition-colors"
                                                @click="
                                                    handleDeleteEntry(entry.id)
                                                "
                                            >
                                                <Trash2 class="w-3 h-3" />
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            </TabsContent>

                            <!-- Access tab -->
                            <TabsContent
                                value="access"
                                class="mt-0 flex-1 overflow-y-auto"
                            >
                                <div class="divide-y divide-eve-border/50">
                                    <div
                                        v-for="access in currentNetwork.accesses"
                                        :key="access.id"
                                        class="group flex items-center gap-3 px-4 py-2.5 hover:bg-eve-bg-1/50 transition-colors"
                                    >
                                        <img
                                            v-if="access.entity"
                                            :src="
                                                getPortraitUrl(
                                                    access.accessible_type,
                                                    access.accessible_id
                                                )
                                            "
                                            class="w-7 h-7 rounded shrink-0"
                                        />
                                        <div
                                            v-else
                                            class="w-7 h-7 rounded bg-eve-bg-2 flex items-center justify-center shrink-0"
                                        >
                                            <Shield
                                                class="w-3.5 h-3.5 text-eve-text-3"
                                            />
                                        </div>

                                        <div class="flex-1 min-w-0">
                                            <div
                                                class="flex items-center gap-1.5"
                                            >
                                                <span
                                                    class="text-xs font-medium text-eve-text-1 truncate"
                                                >
                                                    {{
                                                        access.entity?.name ??
                                                        `#${access.accessible_id}`
                                                    }}
                                                </span>
                                                <span
                                                    v-if="access.entity?.ticker"
                                                    class="text-[10px] text-eve-text-3"
                                                    >[{{
                                                        access.entity.ticker
                                                    }}]</span
                                                >
                                            </div>
                                            <div
                                                v-if="
                                                    access.entity
                                                        ?.corporation ||
                                                    access.entity?.alliance
                                                "
                                                class="text-[10px] text-eve-text-3 mt-0.5 truncate"
                                            >
                                                <span
                                                    v-if="
                                                        access.entity
                                                            ?.corporation
                                                    "
                                                    >[{{
                                                        access.entity
                                                            .corporation.ticker
                                                    }}]
                                                    {{
                                                        access.entity
                                                            .corporation.name
                                                    }}</span
                                                >
                                                <span
                                                    v-if="
                                                        access.entity
                                                            ?.corporation &&
                                                        access.entity?.alliance
                                                    "
                                                >
                                                    ·
                                                </span>
                                                <span
                                                    v-if="
                                                        access.entity?.alliance
                                                    "
                                                    >&lt;{{
                                                        access.entity.alliance
                                                            .ticker
                                                    }}&gt;
                                                    {{
                                                        access.entity.alliance
                                                            .name
                                                    }}</span
                                                >
                                            </div>
                                        </div>

                                        <div
                                            class="flex items-center gap-2 shrink-0"
                                        >
                                            <Badge
                                                variant="outline"
                                                class="text-[9px] h-5"
                                                :class="
                                                    access.is_owner
                                                        ? 'border-eve-cyan/40 text-eve-cyan'
                                                        : ''
                                                "
                                                >{{
                                                    access.is_owner
                                                        ? 'owner'
                                                        : access.permission
                                                }}</Badge
                                            >
                                            <button
                                                v-if="!access.is_owner"
                                                class="p-1 rounded text-eve-text-3 opacity-0 group-hover:opacity-100 hover:text-eve-red hover:bg-eve-red/10 transition-all"
                                                @click="
                                                    handleDeleteAccess(
                                                        access.id
                                                    )
                                                "
                                            >
                                                <Trash2 class="w-3 h-3" />
                                            </button>
                                        </div>
                                    </div>
                                </div>
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
        <Dialog v-model:open="showAnnotationDialog">
            <DialogContent
                class="bg-eve-bg-1 border-eve-border text-eve-text-1 sm:max-w-lg"
                :show-close-button="false"
            >
                <DialogHeader>
                    <DialogTitle class="text-sm text-eve-text-1">
                        Annotation
                    </DialogTitle>
                    <DialogDescription class="text-[10px] text-eve-text-3">
                        Tag a character, corporation, or alliance with intel
                        annotations.
                    </DialogDescription>
                </DialogHeader>

                <div class="space-y-4">
                    <div>
                        <label
                            class="text-[10px] font-semibold tracking-[0.15em] text-eve-text-3 block mb-1.5"
                            >TARGET</label
                        >
                        <EntityCombobox
                            v-model="annotationEntity"
                            placeholder="Search character, corp, or alliance..."
                        />
                    </div>

                    <div>
                        <label
                            class="text-[10px] font-semibold tracking-[0.15em] text-eve-text-3 block mb-1.5"
                            >TAGS</label
                        >
                        <div class="flex flex-wrap gap-1 mb-2">
                            <button
                                v-for="preset in PRESET_ANNOTATION_TAGS"
                                :key="preset.tag"
                                class="px-2 py-0.5 rounded text-[9px] font-bold tracking-wider transition-all cursor-pointer"
                                :style="{
                                    backgroundColor: preset.color + '22',
                                    color: preset.color,
                                    border: parseAnnotationTags(
                                        entryForm.tags
                                    ).includes(preset.tag)
                                        ? `1px solid ${preset.color}`
                                        : '1px solid transparent',
                                }"
                                @click="togglePresetTag(preset.tag)"
                            >
                                {{ preset.tag }}
                            </button>
                        </div>
                        <Input
                            v-model="entryForm.tags"
                            type="text"
                            placeholder="Tags separated by |"
                        />
                    </div>

                    <div>
                        <label
                            class="text-[10px] font-semibold tracking-[0.15em] text-eve-text-3 block mb-1.5"
                            >NOTES (OPTIONAL)</label
                        >
                        <Textarea
                            v-model="entryForm.notes"
                            placeholder="Add a note..."
                            class="min-h-16 font-mono text-xs"
                        />
                    </div>

                    <div
                        v-if="
                            annotationEntity.name &&
                            parseAnnotationTags(entryForm.tags).length > 0
                        "
                        class="flex items-center gap-2 px-3 py-2 rounded-md bg-eve-bg-2/50 border border-eve-border/50"
                    >
                        <span class="text-[10px] text-eve-text-3 shrink-0"
                            >Preview</span
                        >
                        <div class="flex flex-wrap gap-1">
                            <Badge
                                v-for="tag in parseAnnotationTags(
                                    entryForm.tags
                                )"
                                :key="tag"
                                variant="secondary"
                                class="text-[9px] h-4 px-1.5"
                                :style="{
                                    backgroundColor:
                                        (getAnnotationColor(
                                            parseAnnotationTags(entryForm.tags)
                                        ) ?? '#556677') + '33',
                                    color:
                                        getAnnotationColor(
                                            parseAnnotationTags(entryForm.tags)
                                        ) ?? '#556677',
                                }"
                                >{{ tag }}</Badge
                            >
                        </div>
                        <span class="text-[10px] text-eve-text-1 truncate">{{
                            annotationEntity.name
                        }}</span>
                    </div>
                </div>

                <DialogFooter>
                    <Button
                        :disabled="!annotationEntity.id || loading"
                        @click="handleAddEntry"
                    >
                        SAVE
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>

        <!-- ==================== ACCESS DIALOG ==================== -->
        <Dialog v-model:open="showAccessDialog">
            <DialogContent
                class="bg-eve-bg-1 border-eve-border text-eve-text-1 sm:max-w-md"
                :show-close-button="false"
            >
                <DialogHeader>
                    <DialogTitle class="text-sm text-eve-text-1"
                        >Grant Access</DialogTitle
                    >
                    <DialogDescription class="text-[10px] text-eve-text-3">
                        Search for a character, corporation, or alliance to
                        grant network access.
                    </DialogDescription>
                </DialogHeader>

                <div class="space-y-4">
                    <div>
                        <label
                            class="text-[10px] font-semibold tracking-[0.15em] text-eve-text-3 block mb-1.5"
                            >ENTITY</label
                        >
                        <EntityCombobox
                            v-model="accessEntity"
                            placeholder="Search entity..."
                        />
                    </div>

                    <div>
                        <label
                            class="text-[10px] font-semibold tracking-[0.15em] text-eve-text-3 block mb-1.5"
                            >PERMISSION</label
                        >
                        <div class="flex gap-1">
                            <button
                                v-for="p in [
                                    'viewer',
                                    'member',
                                    'manager',
                                ] as const"
                                :key="p"
                                class="flex-1 py-1.5 rounded text-[10px] font-bold tracking-wider transition-colors cursor-pointer"
                                :class="
                                    accessPermission === p
                                        ? 'bg-eve-cyan/20 text-eve-cyan border border-eve-cyan/50'
                                        : 'bg-eve-bg-2 text-eve-text-3 border border-eve-border hover:border-eve-text-3'
                                "
                                @click="accessPermission = p"
                            >
                                {{ p.toUpperCase() }}
                            </button>
                        </div>
                        <p class="text-[9px] text-eve-text-3 mt-1">
                            <template v-if="accessPermission === 'viewer'"
                                >Can view intel entries</template
                            >
                            <template v-else-if="accessPermission === 'member'"
                                >Can view and add/edit entries</template
                            >
                            <template v-else
                                >Full control including access
                                management</template
                            >
                        </p>
                    </div>
                </div>

                <DialogFooter>
                    <Button
                        :disabled="!accessEntity.id || loading"
                        @click="handleAddAccess"
                    >
                        GRANT ACCESS
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
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
