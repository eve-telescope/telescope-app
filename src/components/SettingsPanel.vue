<script setup lang="ts">
import { ref } from 'vue'
import {
    Trash2,
    HelpCircle,
    Database,
    RefreshCw,
    Copy,
    Check,
} from 'lucide-vue-next'
import { invoke } from '@tauri-apps/api/core'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { openExternalUrl } from '../utils/openExternal'
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
const loginError = ref<string | null>(null)

// Shown alongside the button so users can always reach the login page
// manually, even when the system browser opener is broken.
const loginUrl = `${API_BASE_URL}/eve?desktop=1`
const loginUrlCopied = ref(false)

async function loginWithEve() {
    loginError.value = null
    try {
        await openExternalUrl(loginUrl)
    } catch (e) {
        // Surface the failure instead of silently doing nothing — on Linux
        // a broken opener (e.g. env workarounds leaking into xdg-open)
        // previously made this button appear dead.
        console.error('Failed to open EVE login in browser:', e)
        loginError.value = `Could not open the browser: ${e}`
    }
}

async function copyLoginUrl() {
    try {
        await writeText(loginUrl)
        loginUrlCopied.value = true
        setTimeout(() => (loginUrlCopied.value = false), 2000)
    } catch (e) {
        console.error('Failed to copy login link:', e)
    }
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

                        <p v-if="loginError" class="text-xs text-eve-red">
                            {{ loginError }}
                        </p>

                        <div
                            class="rounded border border-eve-border bg-eve-bg-1 px-2 py-1.5"
                        >
                            <p class="mb-1 text-[10px] text-eve-text-3">
                                Or open this link in your browser:
                            </p>
                            <div class="flex items-center gap-1.5">
                                <code
                                    class="flex-1 truncate font-mono text-[10px] text-eve-text-2 select-all"
                                    >{{ loginUrl }}</code
                                >
                                <button
                                    class="shrink-0 rounded p-1 text-eve-text-3 transition-colors hover:bg-eve-bg-hover hover:text-eve-text-1"
                                    :title="
                                        loginUrlCopied ? 'Copied!' : 'Copy link'
                                    "
                                    @click="copyLoginUrl"
                                >
                                    <Check
                                        v-if="loginUrlCopied"
                                        class="w-3 h-3 text-eve-green"
                                    />
                                    <Copy v-else class="w-3 h-3" />
                                </button>
                            </div>
                        </div>

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
