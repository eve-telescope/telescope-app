<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { getVersion } from '@tauri-apps/api/app'
import { openUrl } from '@tauri-apps/plugin-opener'
import { ExternalLink, Github, Globe } from 'lucide-vue-next'
import { useSettings } from '../composables/useSettings'
import { formatShortcut, isMac } from '../utils/shortcut'

const { settings } = useSettings()

const version = ref('...')

const mod = computed(() => (isMac ? '⌘' : 'Ctrl'))

const globalHotkey = computed(() => {
    return formatShortcut(settings.value.globalShortcut)
})

onMounted(async () => {
    version.value = await getVersion()
})

function openExternal(url: string) {
    openUrl(url)
}
</script>

<template>
    <div class="h-screen bg-eve-bg-0 text-eve-text-1 overflow-auto">
        <div class="max-w-2xl mx-auto p-8">
            <!-- Header -->
            <div class="flex items-center gap-4 mb-8">
                <svg
                    class="w-16 h-16 text-eve-cyan drop-shadow-[0_0_15px_rgba(0,212,255,0.3)]"
                    viewBox="0 0 24 24"
                    fill="none"
                >
                    <rect
                        x="0"
                        y="0"
                        width="24"
                        height="24"
                        rx="4.5"
                        fill="#12151a"
                    />
                    <g
                        stroke="currentColor"
                        stroke-width="1.5"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    >
                        <circle cx="12" cy="12" r="7" />
                        <line x1="19" x2="16" y1="12" y2="12" />
                        <line x1="8" x2="5" y1="12" y2="12" />
                        <line x1="12" x2="12" y1="8" y2="5" />
                        <line x1="12" x2="12" y1="19" y2="16" />
                    </g>
                </svg>
                <div>
                    <h1 class="text-2xl font-bold tracking-wider">TELESCOPE</h1>
                    <p class="text-eve-text-3 text-sm">EVE Online Intel Tool</p>
                    <p class="text-eve-text-3 text-xs mt-1">
                        Version {{ version }}
                    </p>
                </div>
            </div>

            <!-- How to Use -->
            <section class="mb-8">
                <h2
                    class="text-xs font-semibold tracking-[0.15em] text-eve-text-3 mb-4"
                >
                    HOW TO USE
                </h2>
                <div class="space-y-4">
                    <!-- Step 1 -->
                    <div class="flex gap-4 items-start">
                        <div
                            class="w-7 h-7 rounded-full bg-eve-cyan/20 text-eve-cyan flex items-center justify-center text-sm font-bold shrink-0"
                        >
                            1
                        </div>
                        <div>
                            <h3 class="font-medium text-sm">
                                Select pilots in Local
                            </h3>
                            <p class="text-eve-text-3 text-xs mt-1">
                                In EVE, open Local chat and select all pilots
                            </p>
                            <kbd
                                class="inline-block mt-2 px-2 py-1 bg-eve-bg-2 rounded text-xs text-eve-text-2"
                                >{{ mod }} + A</kbd
                            >
                        </div>
                    </div>

                    <!-- Step 2 -->
                    <div class="flex gap-4 items-start">
                        <div
                            class="w-7 h-7 rounded-full bg-eve-cyan/20 text-eve-cyan flex items-center justify-center text-sm font-bold shrink-0"
                        >
                            2
                        </div>
                        <div>
                            <h3 class="font-medium text-sm">
                                Copy to clipboard
                            </h3>
                            <p class="text-eve-text-3 text-xs mt-1">
                                Copy the selected pilot names
                            </p>
                            <kbd
                                class="inline-block mt-2 px-2 py-1 bg-eve-bg-2 rounded text-xs text-eve-text-2"
                                >{{ mod }} + C</kbd
                            >
                        </div>
                    </div>

                    <!-- Step 3 -->
                    <div class="flex gap-4 items-start">
                        <div
                            class="w-7 h-7 rounded-full bg-eve-cyan/20 text-eve-cyan flex items-center justify-center text-sm font-bold shrink-0"
                        >
                            3
                        </div>
                        <div>
                            <h3 class="font-medium text-sm">
                                Scan with Telescope
                            </h3>
                            <p class="text-eve-text-3 text-xs mt-1">
                                Press your global hotkey to paste and scan
                                instantly
                            </p>
                            <kbd
                                class="inline-block mt-2 px-2 py-1 bg-eve-cyan/20 border border-eve-cyan/30 rounded text-xs text-eve-cyan"
                                >{{ globalHotkey }}</kbd
                            >
                        </div>
                    </div>
                </div>
            </section>

            <!-- Tips -->
            <section class="mb-8">
                <h2
                    class="text-xs font-semibold tracking-[0.15em] text-eve-text-3 mb-4"
                >
                    TIPS
                </h2>
                <div class="space-y-2 text-sm">
                    <div class="flex gap-2 items-start">
                        <span class="text-eve-cyan">•</span>
                        <p class="text-eve-text-2">
                            Configure a
                            <span class="text-eve-text-1">global hotkey</span>
                            to scan directly from EVE without switching windows
                        </p>
                    </div>
                    <div class="flex gap-2 items-start">
                        <span class="text-eve-cyan">•</span>
                        <p class="text-eve-text-2">
                            Results are
                            <span class="text-eve-text-1">cached</span> —
                            repeated scans are instant
                        </p>
                    </div>
                    <div class="flex gap-2 items-start">
                        <span class="text-eve-cyan">•</span>
                        <p class="text-eve-text-2">
                            Click
                            <span class="text-eve-text-1">Share Scan</span> to
                            copy a link for your fleet
                        </p>
                    </div>
                    <div class="flex gap-2 items-start">
                        <span class="text-eve-cyan">•</span>
                        <p class="text-eve-text-2">
                            Use <span class="text-eve-text-1">filters</span> on
                            the right to narrow down by corp/alliance
                        </p>
                    </div>
                </div>
            </section>

            <!-- Links -->
            <section class="mb-8">
                <h2
                    class="text-xs font-semibold tracking-[0.15em] text-eve-text-3 mb-4"
                >
                    LINKS
                </h2>
                <div class="flex flex-wrap gap-2">
                    <button
                        @click="openExternal('https://eve-telescope.com')"
                        class="flex items-center gap-2 px-4 py-2 bg-eve-bg-1 border border-eve-border rounded-lg text-sm hover:border-eve-cyan hover:text-eve-cyan transition-colors"
                    >
                        <Globe class="w-4 h-4" />
                        Website
                        <ExternalLink class="w-3 h-3 opacity-50" />
                    </button>
                    <button
                        @click="
                            openExternal(
                                'https://github.com/eve-telescope/telescope-app'
                            )
                        "
                        class="flex items-center gap-2 px-4 py-2 bg-eve-bg-1 border border-eve-border rounded-lg text-sm hover:border-eve-cyan hover:text-eve-cyan transition-colors"
                    >
                        <Github class="w-4 h-4" />
                        GitHub
                        <ExternalLink class="w-3 h-3 opacity-50" />
                    </button>
                    <button
                        @click="openExternal('https://zkillboard.com')"
                        class="flex items-center gap-2 px-4 py-2 bg-eve-bg-1 border border-eve-border rounded-lg text-sm hover:border-eve-cyan hover:text-eve-cyan transition-colors"
                    >
                        zKillboard
                        <ExternalLink class="w-3 h-3 opacity-50" />
                    </button>
                </div>
            </section>

            <!-- Credits -->
            <section class="text-center text-eve-text-3 text-xs">
                <p>Data provided by zKillboard and EVE ESI</p>
                <p class="mt-1">EVE Online and all related assets © CCP hf.</p>
                <p class="mt-3 text-eve-text-3/50">
                    Made with ♥ for the EVE community
                </p>
            </section>
        </div>
    </div>
</template>
