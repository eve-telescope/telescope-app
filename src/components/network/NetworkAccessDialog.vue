<script setup lang="ts">
import { ref, watch } from 'vue'
import { Button } from '@/components/ui/button'
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'
import EntityCombobox from '../EntityCombobox.vue'
import type { PermissionLevel } from '../../types'
import {
    describePermission,
    type AccessSavePayload,
    type NetworkEntitySelection,
} from '../../utils/network'

const open = defineModel<boolean>('open', { required: true })

defineProps<{
    loading: boolean
}>()

const emit = defineEmits<{
    save: [payload: AccessSavePayload]
}>()

const accessEntity = ref<NetworkEntitySelection>({
    id: '',
    name: '',
    type: null,
})
const accessPermission = ref<PermissionLevel>('viewer')

watch(open, (isOpen) => {
    if (isOpen) {
        accessEntity.value = { id: '', name: '', type: null }
        accessPermission.value = 'viewer'
    }
})

function handleSave() {
    const entity = accessEntity.value
    if (!entity.id || !entity.type) return
    emit('save', {
        entity: { id: entity.id, name: entity.name, type: entity.type },
        permission: accessPermission.value,
    })
}
</script>

<template>
    <Dialog v-model:open="open">
        <DialogContent
            class="bg-eve-bg-1 border-eve-border text-eve-text-1 sm:max-w-md"
            :show-close-button="false"
        >
            <DialogHeader>
                <DialogTitle class="text-sm text-eve-text-1"
                    >Grant Access</DialogTitle
                >
                <DialogDescription class="text-[10px] text-eve-text-3">
                    Search for a character, corporation, or alliance to grant
                    network access.
                </DialogDescription>
            </DialogHeader>

            <div class="space-y-4">
                <div>
                    <label
                        class="text-[10px] font-semibold tracking-[0.15em] text-eve-text-3 block mb-1.5"
                        >ENTITY</label
                    >
                    <EntityCombobox
                        v-model="accessEntity"
                        placeholder="Search entity..."
                    />
                </div>

                <div>
                    <label
                        class="text-[10px] font-semibold tracking-[0.15em] text-eve-text-3 block mb-1.5"
                        >PERMISSION</label
                    >
                    <div class="flex gap-1">
                        <button
                            v-for="p in [
                                'viewer',
                                'member',
                                'manager',
                            ] as const"
                            :key="p"
                            class="flex-1 py-1.5 rounded text-[10px] font-bold tracking-wider transition-colors cursor-pointer"
                            :class="
                                accessPermission === p
                                    ? 'bg-eve-cyan/20 text-eve-cyan border border-eve-cyan/50'
                                    : 'bg-eve-bg-2 text-eve-text-3 border border-eve-border hover:border-eve-text-3'
                            "
                            @click="accessPermission = p"
                        >
                            {{ p.toUpperCase() }}
                        </button>
                    </div>
                    <p class="text-[9px] text-eve-text-3 mt-1">
                        {{ describePermission(accessPermission) }}
                    </p>
                </div>
            </div>

            <DialogFooter>
                <Button
                    :disabled="!accessEntity.id || loading"
                    @click="handleSave"
                >
                    GRANT ACCESS
                </Button>
            </DialogFooter>
        </DialogContent>
    </Dialog>
</template>
