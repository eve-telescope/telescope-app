<script setup lang="ts">
import { initIntelStore } from './stores/intel'

// Every Tauri window (main '/', overlay, networks, about) boots through
// this shared root. All routes except /about read the intel store, and the
// networks window mounts NetworkManager directly, so this is the one place
// that guarantees every window is hydrated without each component owning
// store lifecycle. initIntelStore() is idempotent, so App.vue's own call
// stays harmless, and the cost for /about is a single no-op state fetch.
void initIntelStore()
</script>

<template>
    <router-view />
</template>
