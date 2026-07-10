<script setup lang="ts">
import { Download, X, ExternalLink } from 'lucide-vue-next'
import { openExternalUrl } from '../utils/openExternal'
import type { UpdateInfo } from '../composables/useUpdateChecker'
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'

const props = defineProps<{
    info: UpdateInfo
}>()

const emit = defineEmits<{
    dismiss: []
}>()

function openRelease() {
    openExternalUrl(props.info.release_url)
}
</script>

<template>
    <Dialog :open="true" @update:open="(open) => !open && emit('dismiss')">
        <DialogContent
            class="max-w-md overflow-hidden p-0"
            :show-close-button="false"
        >
            <DialogHeader class="pr-14">
                <div class="flex items-center gap-3">
                    <div
                        class="w-10 h-10 rounded-lg bg-eve-cyan/20 flex items-center justify-center"
                    >
                        <Download class="w-5 h-5 text-eve-cyan" />
                    </div>
                    <div>
                        <DialogTitle>Update Available</DialogTitle>
                        <DialogDescription>
                            v{{ info.current_version }} → v{{
                                info.latest_version
                            }}
                        </DialogDescription>
                    </div>
                </div>
                <Button
                    variant="ghost"
                    size="icon-sm"
                    class="absolute right-4 top-4"
                    @click="emit('dismiss')"
                >
                    <X class="w-5 h-5" />
                </Button>
            </DialogHeader>

            <div class="p-4 pt-0">
                <p class="text-sm text-eve-text-2 mb-4">
                    A new version of Telescope is available. Update to get the
                    latest features and bug fixes.
                </p>

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

            <DialogFooter class="gap-3 sm:justify-stretch [&>*]:flex-1">
                <Button
                    variant="outline"
                    class="text-sm"
                    @click="emit('dismiss')"
                >
                    Later
                </Button>
                <Button class="text-sm" @click="openRelease">
                    <Download class="w-4 h-4" />
                    Download
                    <ExternalLink class="w-3 h-3 opacity-70" />
                </Button>
            </DialogFooter>
        </DialogContent>
    </Dialog>
</template>
