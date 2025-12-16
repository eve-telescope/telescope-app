import { ref, computed, type Ref } from 'vue'
import type { PilotIntel } from '../types'

export type FilterMode = 'single' | 'multi'

export interface PilotFiltersOptions {
    mode?: FilterMode
}

export function usePilotFilters(
    pilots: Ref<PilotIntel[]>,
    options: PilotFiltersOptions = {}
) {
    const mode = options.mode ?? 'multi'

    const threatFilter = ref<string | null>(null)
    const selectedTags = ref<Set<string>>(new Set())
    const corpFilter = ref<string | null>(null)
    const allianceFilter = ref<string | null>(null)
    const selectedCorps = ref<Set<string>>(new Set())
    const selectedAlliances = ref<Set<string>>(new Set())

    function toggleThreatFilter(level: string) {
        threatFilter.value = threatFilter.value === level ? null : level
    }

    function toggleTag(tag: string) {
        if (mode === 'single') {
            if (selectedTags.value.has(tag)) {
                selectedTags.value = new Set()
            } else {
                selectedTags.value = new Set([tag])
            }
        } else {
            const newSet = new Set(selectedTags.value)
            if (newSet.has(tag)) {
                newSet.delete(tag)
            } else {
                newSet.add(tag)
            }
            selectedTags.value = newSet
        }
    }

    function toggleCorpFilter(ticker: string) {
        if (mode === 'single') {
            corpFilter.value = corpFilter.value === ticker ? null : ticker
        } else {
            const newSet = new Set(selectedCorps.value)
            if (newSet.has(ticker)) {
                newSet.delete(ticker)
            } else {
                newSet.add(ticker)
            }
            selectedCorps.value = newSet
        }
    }

    function toggleAllianceFilter(ticker: string) {
        if (mode === 'single') {
            allianceFilter.value =
                allianceFilter.value === ticker ? null : ticker
        } else {
            const newSet = new Set(selectedAlliances.value)
            if (newSet.has(ticker)) {
                newSet.delete(ticker)
            } else {
                newSet.add(ticker)
            }
            selectedAlliances.value = newSet
        }
    }

    function clearFilters() {
        threatFilter.value = null
        selectedTags.value = new Set()
        corpFilter.value = null
        allianceFilter.value = null
        selectedCorps.value = new Set()
        selectedAlliances.value = new Set()
    }

    const hasFilters = computed(() => {
        return (
            threatFilter.value !== null ||
            selectedTags.value.size > 0 ||
            corpFilter.value !== null ||
            allianceFilter.value !== null ||
            selectedCorps.value.size > 0 ||
            selectedAlliances.value.size > 0
        )
    })

    const filteredPilots = computed(() => {
        let result = pilots.value

        if (threatFilter.value) {
            result = result.filter(
                (p) => p.threat_level.toLowerCase() === threatFilter.value
            )
        }

        if (selectedTags.value.size > 0) {
            result = result.filter((p) => {
                const flags = p.flags
                if (selectedTags.value.has('super') && flags.is_super)
                    return true
                if (selectedTags.value.has('capital') && flags.is_capital)
                    return true
                if (selectedTags.value.has('blops') && flags.is_blops)
                    return true
                if (selectedTags.value.has('recon') && flags.is_recon)
                    return true
                if (selectedTags.value.has('cyno') && flags.is_cyno) return true
                if (selectedTags.value.has('solo') && flags.is_solo) return true
                return false
            })
        }

        if (mode === 'single') {
            if (corpFilter.value) {
                result = result.filter(
                    (p) => p.character.corporation_ticker === corpFilter.value
                )
            }
            if (allianceFilter.value) {
                result = result.filter(
                    (p) => p.character.alliance_ticker === allianceFilter.value
                )
            }
        } else {
            if (selectedCorps.value.size > 0) {
                result = result.filter((p) =>
                    selectedCorps.value.has(
                        p.character.corporation_name || 'Unknown'
                    )
                )
            }
            if (selectedAlliances.value.size > 0) {
                result = result.filter(
                    (p) =>
                        p.character.alliance_name &&
                        selectedAlliances.value.has(p.character.alliance_name)
                )
            }
        }

        return result
    })

    return {
        threatFilter,
        selectedTags,
        corpFilter,
        allianceFilter,
        selectedCorps,
        selectedAlliances,
        hasFilters,
        filteredPilots,
        toggleThreatFilter,
        toggleTag,
        toggleCorpFilter,
        toggleAllianceFilter,
        clearFilters,
    }
}
