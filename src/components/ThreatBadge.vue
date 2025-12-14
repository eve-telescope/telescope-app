<script setup lang="ts">
defineProps<{
    level: string
    compact?: boolean
}>()
</script>

<template>
    <span
        class="inline-block px-1.5 py-0.5 rounded text-[10px] font-bold tracking-wide data-[threat=extreme]:bg-eve-threat-extreme/20 data-[threat=extreme]:text-eve-threat-extreme data-[threat=high]:bg-eve-threat-high/20 data-[threat=high]:text-eve-threat-high data-[threat=moderate]:bg-eve-threat-moderate/20 data-[threat=moderate]:text-eve-threat-moderate data-[threat=low]:bg-eve-threat-low/20 data-[threat=low]:text-eve-threat-low data-[threat=minimal]:bg-eve-threat-minimal/20 data-[threat=minimal]:text-eve-threat-minimal data-[threat=unknown]:bg-eve-threat-unknown/20 data-[threat=unknown]:text-eve-threat-unknown"
        :data-threat="level.toLowerCase()"
    >
        {{
            compact ? level.substring(0, 3).toUpperCase() : level.toUpperCase()
        }}
    </span>
</template>
