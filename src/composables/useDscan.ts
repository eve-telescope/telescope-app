import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { DscanParseResult, SdeStatus } from '../types'

export function useDscan() {
    const rawInput = ref('')
    const result = ref<DscanParseResult | null>(null)
    const sdeStatus = ref<SdeStatus | null>(null)
    const loading = ref(false)
    const syncing = ref(false)
    const error = ref<string | null>(null)

    const shipEntries = computed(
        () => result.value?.entries.filter((entry) => entry.is_ship) ?? []
    )

    async function refreshSdeStatus() {
        try {
            sdeStatus.value = await invoke<SdeStatus>('get_sde_status')
        } catch (err) {
            error.value = String(err)
        }
    }

    async function ensureSdeIndex() {
        syncing.value = true
        error.value = null
        try {
            sdeStatus.value = await invoke<SdeStatus>('ensure_sde_index')
        } catch (err) {
            error.value = String(err)
            await refreshSdeStatus()
        } finally {
            syncing.value = false
        }
    }

    async function parse(text: string) {
        rawInput.value = text
        loading.value = true
        error.value = null
        try {
            result.value = await invoke<DscanParseResult>('parse_dscan', {
                text,
            })
        } catch (err) {
            error.value = String(err)
            result.value = null
        } finally {
            loading.value = false
        }
    }

    function clear() {
        rawInput.value = ''
        result.value = null
        error.value = null
    }

    return {
        rawInput,
        result,
        sdeStatus,
        loading,
        syncing,
        error,
        shipEntries,
        refreshSdeStatus,
        ensureSdeIndex,
        parse,
        clear,
    }
}
