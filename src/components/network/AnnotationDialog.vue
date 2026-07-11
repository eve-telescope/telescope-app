<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { Badge } from '@/components/ui/badge'
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'
import EntityCombobox from '../EntityCombobox.vue'
import type { IntelAnnotation } from '../../types'
import {
    DEFAULT_ANNOTATION_COLOR,
    PRESET_ANNOTATION_TAGS,
    getAnnotationColor,
    parseAnnotationTags,
} from '../../utils/annotations'
import {
    toggleAnnotationTag,
    type AnnotationSavePayload,
    type NetworkEntitySelection,
} from '../../utils/network'

const open = defineModel<boolean>('open', { required: true })

const props = defineProps<{
    loading: boolean
    editing: IntelAnnotation | null
}>()

const emit = defineEmits<{
    save: [payload: AnnotationSavePayload]
}>()

const annotationEntity = ref<NetworkEntitySelection>({
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

const parsedTags = computed(() => parseAnnotationTags(entryForm.value.tags))

const parsedTagSet = computed(() => new Set(parsedTags.value))

const previewColor = computed(
    () => getAnnotationColor(parsedTags.value) ?? DEFAULT_ANNOTATION_COLOR
)

function initForm() {
    const editing = props.editing
    if (editing) {
        annotationEntity.value = {
            id: String(editing.targetId),
            name: editing.targetName,
            type: editing.targetType,
        }
        entryForm.value = {
            tags: editing.tags.join(' | '),
            notes: editing.note ?? '',
        }
    } else {
        annotationEntity.value = { id: '', name: '', type: null }
        entryForm.value = {
            tags: '',
            notes: '',
        }
    }
}

watch(open, (isOpen) => {
    if (isOpen) {
        initForm()
    }
})

function togglePresetTag(tag: string) {
    entryForm.value.tags = toggleAnnotationTag(entryForm.value.tags, tag)
}

function handleSave() {
    const entity = annotationEntity.value
    if (!entity.id || !entity.type) return
    emit('save', {
        editingId: props.editing?.id ?? null,
        entity: { id: entity.id, name: entity.name, type: entity.type },
        tags: parsedTags.value,
        note: entryForm.value.notes || null,
    })
}
</script>

<template>
    <Dialog v-model:open="open">
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
                                border: parsedTagSet.has(preset.tag)
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
                    v-if="annotationEntity.name && parsedTags.length > 0"
                    class="flex items-center gap-2 px-3 py-2 rounded-md bg-eve-bg-2/50 border border-eve-border/50"
                >
                    <span class="text-[10px] text-eve-text-3 shrink-0"
                        >Preview</span
                    >
                    <div class="flex flex-wrap gap-1">
                        <Badge
                            v-for="tag in parsedTags"
                            :key="tag"
                            variant="secondary"
                            class="text-[9px] h-4 px-1.5"
                            :style="{
                                backgroundColor: previewColor + '33',
                                color: previewColor,
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
                    @click="handleSave"
                >
                    SAVE
                </Button>
            </DialogFooter>
        </DialogContent>
    </Dialog>
</template>
