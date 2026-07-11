<script setup lang="ts">
import { computed, ref } from 'vue'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuLabel,
    ContextMenuSeparator,
    ContextMenuSub,
    ContextMenuSubContent,
    ContextMenuSubTrigger,
    ContextMenuTrigger,
} from '@/components/ui/context-menu'
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'
import { User, Building2, Flag } from 'lucide-vue-next'
import { DEFAULT_TAG_COLOR } from '../utils/pilotTags'
import {
    isAuthenticated,
    activeNetwork,
    annotations,
    createAnnotation,
    updateAnnotation,
    removeEntry,
} from '../stores/intel'
import type { EntityType, IntelAnnotation } from '../types'
import {
    DEFAULT_ANNOTATION_COLOR,
    PRESET_ANNOTATION_TAGS,
    getAnnotationColor,
    normalizeAnnotationTag,
    parseAnnotationTags,
} from '../utils/annotations'

const props = defineProps<{
    characterId: number
    characterName: string
    corporationId: number | null
    corporationName: string | null
    allianceId: number | null
    allianceName: string | null
}>()

const PRESET_TAG_SET = new Set(PRESET_ANNOTATION_TAGS.map((p) => p.tag))

// Collect custom tags used across the active network's annotations
const customTags = computed(() => {
    const net = activeNetwork.value
    if (!net) return []
    const tags = new Map<string, string>()
    for (const a of annotations.value) {
        if (a.networkId !== net.id) continue
        for (const tag of a.tags) {
            if (!PRESET_TAG_SET.has(tag) && !tags.has(tag)) {
                tags.set(tag, a.color ?? DEFAULT_TAG_COLOR)
            }
        }
    }
    return [...tags.entries()].map(([tag, color]) => ({ tag, color }))
})

// Find existing annotation for a given entity on the active network
function findAnnotation(
    type: EntityType,
    id: number
): IntelAnnotation | undefined {
    const net = activeNetwork.value
    if (!net) return undefined
    return annotations.value.find(
        (a) =>
            a.networkId === net.id && a.targetType === type && a.targetId === id
    )
}

const characterAnnotation = computed(() =>
    findAnnotation('character', props.characterId)
)
const corporationAnnotation = computed(() =>
    props.corporationId
        ? findAnnotation('corporation', props.corporationId)
        : undefined
)
const allianceAnnotation = computed(() =>
    props.allianceId ? findAnnotation('alliance', props.allianceId) : undefined
)

// Dialog form
const dialogOpen = ref(false)
const adding = ref(false)
const noteForm = ref<{
    annotationId: number | null
    networkId: number
    targetType: EntityType
    targetId: number
    targetName: string
    tags: string
    note: string
} | null>(null)

function openForm(
    targetType: EntityType,
    targetId: number,
    targetName: string,
    existing?: IntelAnnotation
) {
    const net = activeNetwork.value
    if (!net) return
    noteForm.value = {
        annotationId: existing?.id ?? null,
        networkId: net.id,
        targetType,
        targetId,
        targetName,
        tags: existing ? existing.tags.join(' | ') : '',
        note: existing?.note ?? '',
    }
    dialogOpen.value = true
}

function togglePresetTag(tag: string) {
    if (!noteForm.value) return
    const tags = parseAnnotationTags(noteForm.value.tags)
    const normalized = normalizeAnnotationTag(tag)
    const idx = tags.indexOf(normalized)
    if (idx >= 0) {
        tags.splice(idx, 1)
    } else {
        tags.push(normalized)
    }
    noteForm.value.tags = tags.join(' | ')
}

const formTags = computed(() =>
    noteForm.value ? parseAnnotationTags(noteForm.value.tags) : []
)

const formColor = computed(
    () => getAnnotationColor(formTags.value) ?? DEFAULT_ANNOTATION_COLOR
)

async function quickToggle(
    targetType: EntityType,
    targetId: number,
    targetName: string,
    tag: string
) {
    const net = activeNetwork.value
    if (!net) return
    adding.value = true
    try {
        const existing = findAnnotation(targetType, targetId)
        const normalized = normalizeAnnotationTag(tag)
        if (existing) {
            const tags = [...existing.tags]
            const idx = tags.indexOf(normalized)
            if (idx >= 0) {
                tags.splice(idx, 1)
            } else {
                tags.push(normalized)
            }
            if (tags.length === 0) {
                await removeEntry(net.id, existing.id)
            } else {
                await updateAnnotation(
                    net.id,
                    existing.id,
                    targetType,
                    targetId,
                    targetName,
                    tags,
                    existing.note
                )
            }
        } else {
            await createAnnotation(
                net.id,
                targetType,
                targetId,
                targetName,
                [normalized],
                null
            )
        }
    } finally {
        adding.value = false
    }
}

async function submitForm() {
    if (!noteForm.value) return
    const note = noteForm.value.note || null
    const isEmpty = formTags.value.length === 0 && !note
    adding.value = true
    try {
        if (isEmpty && noteForm.value.annotationId != null) {
            await removeEntry(
                noteForm.value.networkId,
                noteForm.value.annotationId
            )
        } else if (!isEmpty && noteForm.value.annotationId != null) {
            await updateAnnotation(
                noteForm.value.networkId,
                noteForm.value.annotationId,
                noteForm.value.targetType,
                noteForm.value.targetId,
                noteForm.value.targetName,
                formTags.value,
                note
            )
        } else if (!isEmpty) {
            await createAnnotation(
                noteForm.value.networkId,
                noteForm.value.targetType,
                noteForm.value.targetId,
                noteForm.value.targetName,
                formTags.value,
                note
            )
        }
        dialogOpen.value = false
        noteForm.value = null
    } finally {
        adding.value = false
    }
}
</script>

<template>
    <Dialog v-model:open="dialogOpen">
        <ContextMenu>
            <ContextMenuTrigger as-child>
                <slot />
            </ContextMenuTrigger>

            <ContextMenuContent class="min-w-56">
                <template v-if="isAuthenticated && activeNetwork">
                    <!-- Character -->
                    <ContextMenuSub>
                        <ContextMenuSubTrigger>
                            <User class="w-4 h-4 text-eve-text-3 shrink-0" />
                            Character
                        </ContextMenuSubTrigger>
                        <ContextMenuSubContent class="min-w-48">
                            <ContextMenuLabel class="flex items-center gap-2">
                                <img
                                    :src="`https://images.evetech.net/characters/${characterId}/portrait?size=32`"
                                    class="w-5 h-5 rounded shrink-0"
                                />
                                <span
                                    class="text-xs text-eve-text-1 truncate"
                                    >{{ characterName }}</span
                                >
                            </ContextMenuLabel>
                            <ContextMenuSeparator />
                            <ContextMenuItem
                                v-for="preset in PRESET_ANNOTATION_TAGS"
                                :key="`char-${preset.tag}`"
                                :disabled="adding"
                                @select.prevent="
                                    quickToggle(
                                        'character',
                                        characterId,
                                        characterName,
                                        preset.tag
                                    )
                                "
                            >
                                <Badge
                                    variant="secondary"
                                    :style="{
                                        backgroundColor: preset.color + '33',
                                        color: preset.color,
                                    }"
                                    >{{ preset.tag }}</Badge
                                >
                                <span
                                    v-if="
                                        characterAnnotation?.tags.includes(
                                            preset.tag
                                        )
                                    "
                                    class="ml-auto text-[9px] text-eve-cyan"
                                    >&#10003;</span
                                >
                            </ContextMenuItem>
                            <template v-if="customTags.length > 0">
                                <ContextMenuSeparator />
                                <ContextMenuItem
                                    v-for="ct in customTags"
                                    :key="`char-custom-${ct.tag}`"
                                    :disabled="adding"
                                    @select.prevent="
                                        quickToggle(
                                            'character',
                                            characterId,
                                            characterName,
                                            ct.tag
                                        )
                                    "
                                >
                                    <Badge
                                        variant="secondary"
                                        :style="{
                                            backgroundColor: ct.color + '33',
                                            color: ct.color,
                                        }"
                                        >{{ ct.tag }}</Badge
                                    >
                                    <span
                                        v-if="
                                            characterAnnotation?.tags.includes(
                                                ct.tag
                                            )
                                        "
                                        class="ml-auto text-[9px] text-eve-cyan"
                                        >&#10003;</span
                                    >
                                </ContextMenuItem>
                            </template>
                            <ContextMenuSeparator />
                            <ContextMenuItem
                                @select.prevent="
                                    openForm(
                                        'character',
                                        characterId,
                                        characterName,
                                        characterAnnotation
                                    )
                                "
                            >
                                Annotation...
                            </ContextMenuItem>
                        </ContextMenuSubContent>
                    </ContextMenuSub>

                    <!-- Corporation -->
                    <ContextMenuSub v-if="corporationId && corporationName">
                        <ContextMenuSubTrigger>
                            <Building2
                                class="w-4 h-4 text-eve-text-3 shrink-0"
                            />
                            Corporation
                        </ContextMenuSubTrigger>
                        <ContextMenuSubContent class="min-w-48">
                            <ContextMenuLabel class="flex items-center gap-2">
                                <img
                                    :src="`https://images.evetech.net/corporations/${corporationId}/logo?size=32`"
                                    class="w-5 h-5 rounded shrink-0"
                                />
                                <span
                                    class="text-xs text-eve-text-1 truncate"
                                    >{{ corporationName }}</span
                                >
                            </ContextMenuLabel>
                            <ContextMenuSeparator />
                            <ContextMenuItem
                                v-for="preset in PRESET_ANNOTATION_TAGS"
                                :key="`corp-${preset.tag}`"
                                :disabled="adding"
                                @select.prevent="
                                    quickToggle(
                                        'corporation',
                                        corporationId!,
                                        corporationName!,
                                        preset.tag
                                    )
                                "
                            >
                                <Badge
                                    variant="secondary"
                                    :style="{
                                        backgroundColor: preset.color + '33',
                                        color: preset.color,
                                    }"
                                    >{{ preset.tag }}</Badge
                                >
                                <span
                                    v-if="
                                        corporationAnnotation?.tags.includes(
                                            preset.tag
                                        )
                                    "
                                    class="ml-auto text-[9px] text-eve-cyan"
                                    >&#10003;</span
                                >
                            </ContextMenuItem>
                            <template v-if="customTags.length > 0">
                                <ContextMenuSeparator />
                                <ContextMenuItem
                                    v-for="ct in customTags"
                                    :key="`corp-custom-${ct.tag}`"
                                    :disabled="adding"
                                    @select.prevent="
                                        quickToggle(
                                            'corporation',
                                            corporationId!,
                                            corporationName!,
                                            ct.tag
                                        )
                                    "
                                >
                                    <Badge
                                        variant="secondary"
                                        :style="{
                                            backgroundColor: ct.color + '33',
                                            color: ct.color,
                                        }"
                                        >{{ ct.tag }}</Badge
                                    >
                                    <span
                                        v-if="
                                            corporationAnnotation?.tags.includes(
                                                ct.tag
                                            )
                                        "
                                        class="ml-auto text-[9px] text-eve-cyan"
                                        >&#10003;</span
                                    >
                                </ContextMenuItem>
                            </template>
                            <ContextMenuSeparator />
                            <ContextMenuItem
                                @select.prevent="
                                    openForm(
                                        'corporation',
                                        corporationId!,
                                        corporationName!,
                                        corporationAnnotation
                                    )
                                "
                            >
                                Annotation...
                            </ContextMenuItem>
                        </ContextMenuSubContent>
                    </ContextMenuSub>

                    <!-- Alliance -->
                    <ContextMenuSub v-if="allianceId && allianceName">
                        <ContextMenuSubTrigger>
                            <Flag class="w-4 h-4 text-eve-text-3 shrink-0" />
                            Alliance
                        </ContextMenuSubTrigger>
                        <ContextMenuSubContent class="min-w-48">
                            <ContextMenuLabel class="flex items-center gap-2">
                                <img
                                    :src="`https://images.evetech.net/alliances/${allianceId}/logo?size=32`"
                                    class="w-5 h-5 rounded shrink-0"
                                />
                                <span
                                    class="text-xs text-eve-text-1 truncate"
                                    >{{ allianceName }}</span
                                >
                            </ContextMenuLabel>
                            <ContextMenuSeparator />
                            <ContextMenuItem
                                v-for="preset in PRESET_ANNOTATION_TAGS"
                                :key="`ally-${preset.tag}`"
                                :disabled="adding"
                                @select.prevent="
                                    quickToggle(
                                        'alliance',
                                        allianceId!,
                                        allianceName!,
                                        preset.tag
                                    )
                                "
                            >
                                <Badge
                                    variant="secondary"
                                    :style="{
                                        backgroundColor: preset.color + '33',
                                        color: preset.color,
                                    }"
                                    >{{ preset.tag }}</Badge
                                >
                                <span
                                    v-if="
                                        allianceAnnotation?.tags.includes(
                                            preset.tag
                                        )
                                    "
                                    class="ml-auto text-[9px] text-eve-cyan"
                                    >&#10003;</span
                                >
                            </ContextMenuItem>
                            <template v-if="customTags.length > 0">
                                <ContextMenuSeparator />
                                <ContextMenuItem
                                    v-for="ct in customTags"
                                    :key="`ally-custom-${ct.tag}`"
                                    :disabled="adding"
                                    @select.prevent="
                                        quickToggle(
                                            'alliance',
                                            allianceId!,
                                            allianceName!,
                                            ct.tag
                                        )
                                    "
                                >
                                    <Badge
                                        variant="secondary"
                                        :style="{
                                            backgroundColor: ct.color + '33',
                                            color: ct.color,
                                        }"
                                        >{{ ct.tag }}</Badge
                                    >
                                    <span
                                        v-if="
                                            allianceAnnotation?.tags.includes(
                                                ct.tag
                                            )
                                        "
                                        class="ml-auto text-[9px] text-eve-cyan"
                                        >&#10003;</span
                                    >
                                </ContextMenuItem>
                            </template>
                            <ContextMenuSeparator />
                            <ContextMenuItem
                                @select.prevent="
                                    openForm(
                                        'alliance',
                                        allianceId!,
                                        allianceName!,
                                        allianceAnnotation
                                    )
                                "
                            >
                                Annotation...
                            </ContextMenuItem>
                        </ContextMenuSubContent>
                    </ContextMenuSub>
                </template>

                <ContextMenuItem v-else disabled>
                    No active network
                </ContextMenuItem>
            </ContextMenuContent>
        </ContextMenu>

        <!-- Annotation dialog -->
        <DialogContent
            class="bg-eve-bg-1 border-eve-border text-eve-text-1 sm:max-w-sm"
            :show-close-button="false"
        >
            <DialogHeader>
                <DialogTitle class="text-sm">Annotation</DialogTitle>
                <DialogDescription class="text-[10px] text-eve-text-3">
                    {{ noteForm?.targetName }}
                    <span class="uppercase">({{ noteForm?.targetType }})</span>
                </DialogDescription>
            </DialogHeader>

            <div class="space-y-3">
                <div>
                    <div
                        class="text-[10px] font-semibold tracking-[0.15em] text-eve-text-3 mb-1.5"
                    >
                        TAGS
                    </div>
                    <div class="flex flex-wrap gap-1 mb-2">
                        <button
                            v-for="preset in PRESET_ANNOTATION_TAGS"
                            :key="preset.tag"
                            type="button"
                            class="px-2 py-0.5 rounded text-[9px] font-bold tracking-wider transition-all cursor-pointer"
                            :style="{
                                backgroundColor: preset.color + '22',
                                color: preset.color,
                                border: formTags.includes(preset.tag)
                                    ? `1px solid ${preset.color}`
                                    : '1px solid transparent',
                            }"
                            @click="togglePresetTag(preset.tag)"
                        >
                            {{ preset.tag }}
                        </button>
                    </div>
                    <Input
                        v-if="noteForm"
                        v-model="noteForm.tags"
                        placeholder="Tags separated by |"
                    />
                </div>

                <div>
                    <div
                        class="text-[10px] font-semibold tracking-[0.15em] text-eve-text-3 mb-1.5"
                    >
                        NOTE (OPTIONAL)
                    </div>
                    <Textarea
                        v-if="noteForm"
                        v-model="noteForm.note"
                        placeholder="Add a note..."
                        class="min-h-16 font-mono text-xs"
                    />
                </div>

                <div
                    v-if="formTags.length > 0"
                    class="flex items-center gap-2 px-3 py-2 rounded-md bg-eve-bg-2/50 border border-eve-border/50"
                >
                    <span class="text-[10px] text-eve-text-3 shrink-0"
                        >Preview</span
                    >
                    <div class="flex flex-wrap gap-1">
                        <Badge
                            v-for="tag in formTags"
                            :key="tag"
                            variant="secondary"
                            class="text-[9px] h-4 px-1.5"
                            :style="{
                                backgroundColor: formColor + '33',
                                color: formColor,
                            }"
                            >{{ tag }}</Badge
                        >
                    </div>
                </div>
            </div>

            <DialogFooter>
                <Button :disabled="adding" @click="submitForm"> Save </Button>
            </DialogFooter>
        </DialogContent>
    </Dialog>
</template>
