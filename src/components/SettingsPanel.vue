<script setup lang="ts">
import { ref } from 'vue'
import { Trash2, HelpCircle, Database, RefreshCw } from 'lucide-vue-next'
import { invoke } from '@tauri-apps/api/core'
import { openUrl } from '@tauri-apps/plugin-opener'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import ShortcutEditor from './ShortcutEditor.vue'
import { openAboutWindow } from '../composables/useAboutWindow'
import type { SdeStatus } from '../types'
import { API_BASE_URL } from '../utils/config'
import { isAuthenticated, logoutIntel, setApiToken } from '../stores/intel'

const props = defineProps<{
    shortcut: string
    sdeStatus: SdeStatus | null
    sdeSyncing: boolean
}>()

const emit = defineEmits<{
    updateShortcut: [shortcut: string]
    refreshSde: []
}>()

const tokenInput = ref('')
const showTokenInput = ref(false)

async function loginWithEve() {
    await openUrl(`${API_BASE_URL}/eve?desktop=1`)
}

async function submitToken() {
    const token = tokenInput.value.trim()
    if (!token) return
    await setApiToken(token)
    tokenInput.value = ''
    showTokenInput.value = false
}

async function clearCache() {
    try {
        await invoke('clear_cache')
    } catch (e) {
        console.error('Failed to clear cache:', e)
    }
}
</script>

<template>
    <section class="flex-1 overflow-auto bg-eve-bg-0">
        <div class="flex min-h-full w-full flex-col">
            <div class="border-b border-eve-border px-5 py-4">
                <h2 class="text-sm font-bold tracking-[0.18em] text-eve-text-1">
                    GENERAL SETTINGS
                </h2>
                <p class="mt-1 text-xs text-eve-text-3">
                    Configure shortcut behavior and utility actions for
                    Telescope.
                </p>
            </div>

            <div class="border-b border-eve-border px-5 py-4">
                <ShortcutEditor
                    :shortcut="shortcut"
                    @update="emit('updateShortcut', $event)"
                />
            </div>

            <div class="grid md:grid-cols-2">
                <div class="border-b border-eve-border px-5 py-4 md:border-r">
                    <h3
                        class="text-[10px] font-semibold tracking-[0.15em] text-eve-text-3"
                    >
                        INTEL ACCOUNT
                    </h3>
                    <p class="mt-2 text-xs text-eve-text-2">
                        Connect or disconnect your character for network access.
                    </p>

                    <div v-if="isAuthenticated" class="mt-4">
                        <Button variant="outline" @click="logoutIntel">
                            Disconnect Character
                        </Button>
                    </div>

                    <div v-else class="mt-4 space-y-2">
                        <Button
                            variant="outline"
                            class="w-full"
                            @click="loginWithEve"
                        >
                            Connect with EVE
                        </Button>

                        <div v-if="showTokenInput" class="space-y-2">
                            <Input
                                v-model="tokenInput"
                                type="text"
                                placeholder="Paste token..."
                                class="font-mono text-[10px]"
                                @keydown.enter="submitToken"
                            />
                            <Button
                                class="w-full"
                                :disabled="!tokenInput.trim()"
                                @click="submitToken"
                            >
                                Connect
                            </Button>
                        </div>

                        <Button
                            v-else
                            variant="ghost"
                            size="sm"
                            class="w-full"
                            @click="showTokenInput = true"
                        >
                            Paste token manually
                        </Button>
                    </div>
                </div>

                <div class="border-b border-eve-border px-5 py-4">
                    <h3
                        class="text-[10px] font-semibold tracking-[0.15em] text-eve-text-3"
                    >
                        CACHE
                    </h3>
                    <p class="mt-2 text-xs text-eve-text-2">
                        Clear locally cached data if you need to force a
                        refresh.
                    </p>
                    <Button variant="outline" class="mt-4" @click="clearCache">
                        <Trash2 class="w-3.5 h-3.5" />
                        Clear Cache
                    </Button>
                </div>

                <div class="border-b border-eve-border px-5 py-4 md:border-r">
                    <div class="flex items-center justify-between gap-3">
                        <div>
                            <h3
                                class="text-[10px] font-semibold tracking-[0.15em] text-eve-text-3"
                            >
                                SDE CACHE
                            </h3>
                            <p class="mt-2 text-xs text-eve-text-2">
                                Keep the D-scan classification data synced with
                                the latest EVE static export.
                            </p>
                        </div>
                        <Badge variant="outline" class="gap-1">
                            <Database class="w-3 h-3" />
                            {{
                                props.sdeStatus?.build_number
                                    ? `Build ${props.sdeStatus.build_number}`
                                    : 'Not cached'
                            }}
                        </Badge>
                    </div>

                    <p
                        v-if="
                            props.sdeStatus?.latest_build_number &&
                            props.sdeStatus.latest_build_number !==
                                props.sdeStatus.build_number
                        "
                        class="mt-2 text-xs text-eve-cyan"
                    >
                        Latest available build:
                        {{ props.sdeStatus.latest_build_number }}
                    </p>

                    <Button
                        variant="outline"
                        class="mt-4"
                        :disabled="props.sdeSyncing"
                        @click="emit('refreshSde')"
                    >
                        <RefreshCw
                            class="w-3.5 h-3.5"
                            :class="props.sdeSyncing ? 'animate-spin' : ''"
                        />
                        Refresh SDE Cache
                    </Button>
                </div>

                <div class="border-b border-eve-border px-5 py-4">
                    <h3
                        class="text-[10px] font-semibold tracking-[0.15em] text-eve-text-3"
                    >
                        HELP
                    </h3>
                    <p class="mt-2 text-xs text-eve-text-2">
                        Open the about window for version info, usage details,
                        and support links.
                    </p>
                    <Button
                        variant="outline"
                        class="mt-4"
                        @click="openAboutWindow"
                    >
                        <HelpCircle class="w-3.5 h-3.5" />
                        About Telescope
                    </Button>
                </div>
            </div>
        </div>
    </section>
</template>
