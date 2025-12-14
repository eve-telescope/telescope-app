import { ref, computed, reactive } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { PilotIntel } from '../types'

export interface LookupProgress {
    current: number
    total: number
    cache_hits: number
}

interface PilotResult {
    pilot: PilotIntel
    index: number
}

const THREAT_ORDER: Record<string, number> = {
    EXTREME: 0,
    HIGH: 1,
    MODERATE: 2,
    LOW: 3,
    MINIMAL: 4,
    Unknown: 5,
}

function insertSorted(pilots: PilotIntel[], pilot: PilotIntel): PilotIntel[] {
    if (pilots.some((p) => p.character.id === pilot.character.id)) {
        return pilots
    }

    const newPilots = [...pilots]
    const pilotOrder = THREAT_ORDER[pilot.threat_level] ?? 5

    let insertIndex = newPilots.findIndex(
        (p) => (THREAT_ORDER[p.threat_level] ?? 5) > pilotOrder
    )

    if (insertIndex === -1) {
        insertIndex = newPilots.length
    }

    newPilots.splice(insertIndex, 0, pilot)
    return newPilots
}

export function usePilots() {
    const pilotNames = ref('')
    const pilots = ref<PilotIntel[]>([])
    const loading = ref(false)
    const error = ref<string | null>(null)
    const progress = ref<LookupProgress | null>(null)

    const selectedCorps = reactive(new Set<string>())
    const selectedAlliances = reactive(new Set<string>())

    let unlistenProgress: UnlistenFn | null = null
    let unlistenPilotResult: UnlistenFn | null = null
    let listenersSetup = false

    async function setupListeners() {
        if (listenersSetup) return

        unlistenProgress?.()
        unlistenPilotResult?.()

        unlistenProgress = await listen<LookupProgress>(
            'lookup-progress',
            (event) => {
                progress.value = event.payload
            }
        )

        unlistenPilotResult = await listen<PilotResult>(
            'pilot-result',
            (event) => {
                pilots.value = insertSorted(pilots.value, event.payload.pilot)
            }
        )

        listenersSetup = true
    }

    setupListeners()

    const pilotCount = computed(() => {
        return pilotNames.value.split('\n').filter((n) => n.trim()).length
    })

    const filteredPilots = computed(() => {
        if (selectedCorps.size === 0 && selectedAlliances.size === 0) {
            return pilots.value
        }

        return pilots.value.filter((p) => {
            const corpMatch =
                selectedCorps.size === 0 ||
                selectedCorps.has(p.character.corporation_name || 'Unknown')
            const allianceMatch =
                selectedAlliances.size === 0 ||
                (p.character.alliance_name &&
                    selectedAlliances.has(p.character.alliance_name))

            return corpMatch && allianceMatch
        })
    })

    async function lookupPilots(namesOverride?: string) {
        const names = namesOverride ?? pilotNames.value
        if (!names.trim()) return

        if (namesOverride) {
            pilotNames.value = namesOverride
        }

        loading.value = true
        pilots.value = []
        error.value = null
        progress.value = null
        clearFilters()

        try {
            await invoke('lookup_pilots', {
                namesText: names,
            })
        } catch (e) {
            console.error('Failed to lookup pilots:', e)
            error.value = String(e)
        } finally {
            loading.value = false
            progress.value = null
        }
    }

    function toggleCorp(name: string) {
        if (selectedCorps.has(name)) {
            selectedCorps.delete(name)
        } else {
            selectedCorps.add(name)
        }
    }

    function toggleAlliance(name: string) {
        if (selectedAlliances.has(name)) {
            selectedAlliances.delete(name)
        } else {
            selectedAlliances.add(name)
        }
    }

    function clearFilters() {
        selectedCorps.clear()
        selectedAlliances.clear()
    }

    function clear() {
        pilotNames.value = ''
        pilots.value = []
        error.value = null
        progress.value = null
        clearFilters()
    }

    function cleanup() {
        unlistenProgress?.()
        unlistenPilotResult?.()
    }

    return {
        pilotNames,
        pilots,
        filteredPilots,
        loading,
        error,
        progress,
        pilotCount,
        selectedCorps,
        selectedAlliances,
        lookupPilots,
        toggleCorp,
        toggleAlliance,
        clearFilters,
        clear,
        cleanup,
    }
}
