<script setup lang="ts">
import { Download, X, ExternalLink } from 'lucide-vue-next'
import { openUrl } from '@tauri-apps/plugin-opener'
import type { UpdateInfo } from '../composables/useUpdateChecker'

const props = defineProps<{
    info: UpdateInfo
}>()

const emit = defineEmits<{
    dismiss: []
}>()

function openRelease() {
    openUrl(props.info.release_url)
}
</script>

<template>
    <div
        class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50"
        @click.self="emit('dismiss')"
    >
        <div
            class="bg-eve-bg-1 border border-eve-border rounded-lg shadow-2xl max-w-md w-full mx-4 overflow-hidden"
        >
            <!-- Header -->
            <div
                class="flex items-center justify-between p-4 border-b border-eve-border"
            >
                <div class="flex items-center gap-3">
                    <div
                        class="w-10 h-10 rounded-lg bg-eve-cyan/20 flex items-center justify-center"
                    >
                        <Download class="w-5 h-5 text-eve-cyan" />
                    </div>
                    <div>
                        <h2 class="font-semibold text-eve-text-1">
                            Update Available
                        </h2>
                        <p class="text-xs text-eve-text-3">
                            v{{ info.current_version }} → v{{
                                info.latest_version
                            }}
                        </p>
                    </div>
                </div>
                <button
                    class="p-1.5 rounded text-eve-text-3 hover:text-eve-text-1 hover:bg-eve-bg-2 transition-colors"
                    @click="emit('dismiss')"
                >
                    <X class="w-5 h-5" />
                </button>
            </div>

            <!-- Content -->
            <div class="p-4">
                <p class="text-sm text-eve-text-2 mb-4">
                    A new version of Telescope is available. Update to get the
                    latest features and bug fixes.
                </p>

                <!-- Release notes preview -->
                <div
                    v-if="info.release_notes"
                    class="bg-eve-bg-2 rounded-lg p-3 mb-4 max-h-32 overflow-auto text-xs text-eve-text-3"
                >
                    <p class="whitespace-pre-wrap">
                        {{
                            info.release_notes.slice(0, 300) +
                            (info.release_notes.length > 300 ? '...' : '')
                        }}
                    </p>
                </div>
            </div>

            <!-- Actions -->
            <div
                class="flex gap-3 p-4 border-t border-eve-border bg-eve-bg-0/50"
            >
                <button
                    class="flex-1 py-2.5 px-4 bg-eve-bg-2 border border-eve-border rounded text-sm text-eve-text-2 hover:border-eve-text-3 hover:text-eve-text-1 transition-colors"
                    @click="emit('dismiss')"
                >
                    Later
                </button>
                <button
                    class="flex-1 py-2.5 px-4 bg-eve-cyan rounded text-sm font-medium text-eve-bg-0 hover:bg-eve-cyan-dim transition-colors flex items-center justify-center gap-2"
                    @click="openRelease"
                >
                    <Download class="w-4 h-4" />
                    Download
                    <ExternalLink class="w-3 h-3 opacity-70" />
                </button>
            </div>
        </div>
    </div>
</template>
