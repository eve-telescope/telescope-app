<script setup lang="ts">
interface GroupItem {
    id: number
    name: string
    ticker: string
    count: number
}

defineProps<{
    title: string
    items: GroupItem[]
    selected: Set<string>
    getLogoUrl: (id: number) => string
}>()

const emit = defineEmits<{
    toggle: [name: string]
}>()
</script>

<template>
    <div class="flex flex-col min-h-0">
        <h4
            class="text-[10px] font-semibold tracking-wider text-eve-text-3 uppercase mb-2 flex items-center gap-2 shrink-0"
        >
            {{ title }}
            <span
                class="font-mono text-[9px] text-eve-text-3 bg-eve-bg-3 px-1.5 py-0.5 rounded"
                >{{ items.length }}</span
            >
        </h4>
        <div class="flex flex-col gap-0.5 overflow-y-auto flex-1">
            <button
                v-for="item in items"
                :key="item.name"
                class="flex items-center gap-2 px-2 py-1.5 bg-transparent border border-transparent rounded text-eve-text-1 text-[11px] text-left cursor-pointer transition-colors hover:bg-eve-bg-hover shrink-0 data-[active=true]:bg-eve-cyan/10 data-[active=true]:border-eve-cyan-dim data-[active=true]:text-eve-cyan"
                :data-active="selected.has(item.name)"
                :title="item.name"
                @click="emit('toggle', item.name)"
            >
                <img
                    v-if="item.id"
                    :src="getLogoUrl(item.id)"
                    class="w-[22px] h-[22px] rounded bg-eve-bg-3 shrink-0"
                    loading="lazy"
                />
                <span
                    v-if="item.ticker"
                    class="font-mono text-[10px] shrink-0 data-[active=true]:text-eve-cyan data-[active=false]:text-eve-text-2"
                    :data-active="selected.has(item.name)"
                    >[{{ item.ticker }}]</span
                >
                <span class="flex-1 truncate">{{ item.name }}</span>
                <span
                    class="font-mono text-[10px] px-1.5 py-0.5 rounded shrink-0 data-[active=true]:bg-eve-cyan/20 data-[active=true]:text-eve-cyan data-[active=false]:bg-eve-bg-3 data-[active=false]:text-eve-text-3"
                    :data-active="selected.has(item.name)"
                    >{{ item.count }}</span
                >
            </button>
        </div>
    </div>
</template>
