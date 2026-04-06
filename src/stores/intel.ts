import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type {
    PilotIntel,
    IntelNetwork,
    IntelEntry,
    IntelAnnotation,
    IntelEntryDetail,
    ResolvedIntelAnnotation,
    NetworkDetail,
    NetworkAccess,
    NetworkScan,
    PaginatedScans,
    EntityType,
    PermissionLevel,
    SearchResult,
} from '../types'
import { API_BASE_URL } from '../utils/config'
import {
    getAnnotationColor,
    getAnnotationTargetKey,
    mapIntelEntryToAnnotation,
    serializeAnnotationTags,
} from '../utils/annotations'

// ---------------------------------------------------------------------------
// State — single reactive ref, updated by Rust events
// ---------------------------------------------------------------------------

interface IntelState {
    api_base_url: string
    api_token: string | null
    networks: IntelNetwork[]
    entries: IntelEntry[]
    selected_network: NetworkDetail | null
    active_network_ids: number[]
}

function mapEntryToDetail(entry: IntelEntry): IntelEntryDetail {
    return {
        id: entry.id,
        entity_type: entry.entity_type,
        entity_id: entry.entity_id,
        entity_name: entry.entity_name,
        color: entry.color,
        label: entry.label,
        notes: entry.notes,
        added_by: null,
    }
}

function upsertSelectedNetworkEntry(entry: IntelEntry) {
    const selected = state.value.selected_network
    if (!selected || selected.id !== entry.intel_network_id) {
        return selected
    }

    const detail = mapEntryToDetail(entry)
    const existingIndex = selected.entries.findIndex(
        (current) => current.id === entry.id
    )
    const entries = [...selected.entries]

    if (existingIndex >= 0) {
        entries[existingIndex] = detail
    } else {
        entries.unshift(detail)
    }

    return { ...selected, entries }
}

const state = ref<IntelState>({
    api_base_url: API_BASE_URL,
    api_token: null,
    networks: [],
    entries: [],
    selected_network: null,
    active_network_ids: [],
})

// Listen for state changes from Rust — this is how all windows stay in sync
listen<IntelState>('intel-state-changed', (event) => {
    state.value = event.payload
})

// Set the API base URL in Rust, load initial state, and fetch networks if authenticated
invoke('set_api_base_url', { url: API_BASE_URL }).then(() =>
    invoke<IntelState>('get_intel_state').then((s) => {
        state.value = s
        if (s.api_token) {
            invoke('fetch_networks')
        }
    })
)

// ---------------------------------------------------------------------------
// Reactive computed getters
// ---------------------------------------------------------------------------

export const isAuthenticated = computed(() => !!state.value.api_token)
export const networks = computed(() => state.value.networks)
export const entries = computed(() => state.value.entries)
export const selectedNetwork = computed(() => state.value.selected_network)
export const activeNetworkIds = computed(() => state.value.active_network_ids)
export const activeNetworkId = computed(
    () => state.value.active_network_ids[0] ?? null
)
export const annotations = computed<IntelAnnotation[]>(() =>
    state.value.entries.map((entry) => mapIntelEntryToAnnotation(entry))
)
export const selectedNetworkAnnotations = computed<IntelAnnotation[]>(() => {
    const selected = state.value.selected_network
    if (!selected) {
        return []
    }

    return selected.entries.map((entry) =>
        mapIntelEntryToAnnotation(entry, selected.id, selected.name)
    )
})

export const activeNetworks = computed(() =>
    state.value.networks.filter((n) =>
        state.value.active_network_ids.includes(n.id)
    )
)
export const activeNetwork = computed(
    () =>
        state.value.networks.find((n) => n.id === activeNetworkId.value) ?? null
)

export const annotationsByTargetKey = computed<
    Record<string, IntelAnnotation[]>
>(() => {
    const index: Record<string, IntelAnnotation[]> = {}

    for (const annotation of annotations.value) {
        const key = getAnnotationTargetKey(
            annotation.targetType,
            annotation.targetId
        )
        index[key] ??= []
        index[key].push(annotation)
    }

    return index
})

export function getIntelForPilot(
    characterId: number,
    corpId?: number | null,
    allianceId?: number | null
): IntelEntry[] {
    const ids = [characterId, corpId, allianceId].filter(
        (id): id is number => id != null
    )
    return state.value.entries.filter((e) => ids.includes(e.entity_id))
}

export function getAnnotationsForPilot(
    characterId: number,
    corpId?: number | null,
    allianceId?: number | null
): ResolvedIntelAnnotation[] {
    const matches: ResolvedIntelAnnotation[] = []
    const targets: Array<{ scope: EntityType; id: number | null | undefined }> =
        [
            { scope: 'character', id: characterId },
            { scope: 'corporation', id: corpId },
            { scope: 'alliance', id: allianceId },
        ]

    for (const target of targets) {
        if (!target.id) {
            continue
        }

        const key = getAnnotationTargetKey(target.scope, target.id)
        const targetAnnotations = annotationsByTargetKey.value[key] ?? []

        for (const annotation of targetAnnotations) {
            matches.push({
                key: `${target.scope}:${annotation.id}`,
                scope: target.scope,
                annotation,
            })
        }
    }

    return matches
}

export function resolvePilotAnnotations(
    pilot: Pick<PilotIntel, 'character'>
): ResolvedIntelAnnotation[] {
    return getAnnotationsForPilot(
        pilot.character.id,
        pilot.character.corporation_id,
        pilot.character.alliance_id
    )
}

// ---------------------------------------------------------------------------
// Actions — thin invoke() wrappers, Rust handles state + events
// ---------------------------------------------------------------------------

export async function setApiToken(token: string) {
    await invoke('set_api_token', { token })
}

export async function logoutIntel() {
    await invoke('logout_intel')
}

export async function setActiveNetworkIds(ids: number[]) {
    await invoke('set_active_network_ids', { ids })
}

export async function setActiveNetworkId(id: number | null) {
    await setActiveNetworkIds(id == null ? [] : [id])

    if (id == null) {
        await clearSelectedNetwork()
        return
    }

    await selectNetwork(id)
}

export async function fetchNetworks() {
    await invoke('fetch_networks')

    if (activeNetworkId.value != null) {
        const exists = state.value.networks.some(
            (network) => network.id === activeNetworkId.value
        )

        if (exists) {
            await selectNetwork(activeNetworkId.value)
        } else {
            await setActiveNetworkIds([])
            await clearSelectedNetwork()
        }
    }
}

export async function createNetwork(name: string) {
    return invoke<IntelNetwork>('create_network', { name })
}

export async function deleteNetwork(networkId: number) {
    await invoke('delete_network', { networkId })
}

export async function selectNetwork(networkId: number) {
    return invoke<NetworkDetail>('select_network', { networkId })
}

export async function clearSelectedNetwork() {
    await invoke('clear_selected_network')
}

export async function lookupIntel(entityIds: number[]) {
    await invoke('lookup_intel', { entityIds })
}

export async function addEntry(
    networkId: number,
    entityType: EntityType,
    entityId: number,
    entityName: string,
    color: string,
    label: string,
    notes?: string | null
) {
    return invoke<IntelEntry>('add_intel_entry', {
        networkId,
        entityType,
        entityId,
        entityName,
        color,
        label,
        notes: notes ?? null,
    })
}

export async function createAnnotation(
    networkId: number,
    targetType: EntityType,
    targetId: number,
    targetName: string,
    tags: string[],
    note?: string | null
) {
    const label = serializeAnnotationTags(tags)
    return addEntry(
        networkId,
        targetType,
        targetId,
        targetName,
        getAnnotationColor(tags) ?? '#556677',
        label,
        note ?? null
    )
}

export async function updateAnnotation(
    networkId: number,
    annotationId: number,
    targetType: EntityType,
    targetId: number,
    targetName: string,
    tags: string[],
    note?: string | null
) {
    return invoke<IntelEntry>('update_intel_entry', {
        networkId,
        entryId: annotationId,
        entityType: targetType,
        entityId: targetId,
        entityName: targetName,
        color: getAnnotationColor(tags) ?? '#556677',
        label: serializeAnnotationTags(tags),
        notes: note ?? null,
    })
}

export async function removeEntry(networkId: number, entryId: number) {
    await invoke('remove_intel_entry', { networkId, entryId })
}

export function clearEntries() {
    // Local clear for scan reset — Rust will emit updated state
    state.value = { ...state.value, entries: [] }
}

export async function addAccess(
    networkId: number,
    accessibleType: EntityType,
    accessibleId: number,
    accessibleName: string,
    permission: PermissionLevel
) {
    return invoke<NetworkAccess>('add_network_access', {
        networkId,
        accessibleType,
        accessibleId,
        accessibleName,
        permission,
    })
}

export async function removeAccess(networkId: number, accessId: number) {
    await invoke('remove_network_access', { networkId, accessId })
}

export async function searchEntities(query: string, category?: EntityType) {
    return invoke<SearchResult[]>('search_entities', {
        query,
        category: category ?? null,
    })
}

// ---------------------------------------------------------------------------
// Network scans
// ---------------------------------------------------------------------------

export const latestSharedScan = ref<NetworkScan | null>(null)

export async function shareScan(
    networkId: number,
    scanType: 'local' | 'dscan',
    rawText: string,
    solarSystem?: string | null
) {
    return invoke<NetworkScan>('share_scan', {
        networkId,
        scanType,
        rawText,
        solarSystem: solarSystem ?? null,
    })
}

export async function fetchNetworkScans(networkId: number, page: number = 1) {
    return invoke<PaginatedScans>('fetch_network_scans', {
        networkId,
        page,
    })
}

export async function fetchNetworkScan(networkId: number, scanId: number) {
    return invoke<NetworkScan>('fetch_network_scan', {
        networkId,
        scanId,
    })
}

export function onScanShared(scan: NetworkScan) {
    latestSharedScan.value = scan
}

// ---------------------------------------------------------------------------
// Echo real-time handlers — update via Rust state mutation
// ---------------------------------------------------------------------------

export function onEntryCreated(entry: IntelEntry) {
    const current = [...state.value.entries]
    const idx = current.findIndex((e) => e.id === entry.id)
    if (idx >= 0) {
        current[idx] = entry
    } else {
        current.push(entry)
    }
    state.value = {
        ...state.value,
        entries: current,
        selected_network: upsertSelectedNetworkEntry(entry),
    }
}

export function onEntryUpdated(entry: IntelEntry) {
    state.value = {
        ...state.value,
        entries: state.value.entries.map((e) =>
            e.id === entry.id ? entry : e
        ),
        selected_network: upsertSelectedNetworkEntry(entry),
    }
}

export function onEntryDeleted(entryId: number) {
    const sel = state.value.selected_network
    state.value = {
        ...state.value,
        entries: state.value.entries.filter((e) => e.id !== entryId),
        selected_network: sel
            ? { ...sel, entries: sel.entries.filter((e) => e.id !== entryId) }
            : null,
    }
}
