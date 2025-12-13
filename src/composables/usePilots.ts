import { ref, computed, reactive } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { PilotIntel } from "../types";

export function usePilots() {
  const pilotNames = ref("");
  const pilots = ref<PilotIntel[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  const selectedCorps = reactive(new Set<string>());
  const selectedAlliances = reactive(new Set<string>());

  const pilotCount = computed(() => {
    return pilotNames.value.split("\n").filter((n) => n.trim()).length;
  });

  const filteredPilots = computed(() => {
    if (selectedCorps.size === 0 && selectedAlliances.size === 0) {
      return pilots.value;
    }

    return pilots.value.filter((p) => {
      const corpMatch =
        selectedCorps.size === 0 ||
        selectedCorps.has(p.character.corporation_name || "Unknown");
      const allianceMatch =
        selectedAlliances.size === 0 ||
        (p.character.alliance_name && selectedAlliances.has(p.character.alliance_name));

      return corpMatch && allianceMatch;
    });
  });

  async function lookupPilots(namesOverride?: string) {
    const names = namesOverride ?? pilotNames.value;
    if (!names.trim()) return;

    if (namesOverride) {
      pilotNames.value = namesOverride;
    }

    loading.value = true;
    pilots.value = [];
    error.value = null;
    clearFilters();

    try {
      pilots.value = await invoke("lookup_pilots", {
        namesText: names,
      });
    } catch (e) {
      console.error("Failed to lookup pilots:", e);
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  function toggleCorp(name: string) {
    if (selectedCorps.has(name)) {
      selectedCorps.delete(name);
    } else {
      selectedCorps.add(name);
    }
  }

  function toggleAlliance(name: string) {
    if (selectedAlliances.has(name)) {
      selectedAlliances.delete(name);
    } else {
      selectedAlliances.add(name);
    }
  }

  function clearFilters() {
    selectedCorps.clear();
    selectedAlliances.clear();
  }

  function clear() {
    pilotNames.value = "";
    pilots.value = [];
    error.value = null;
    clearFilters();
  }

  return {
    pilotNames,
    pilots,
    filteredPilots,
    loading,
    error,
    pilotCount,
    selectedCorps,
    selectedAlliances,
    lookupPilots,
    toggleCorp,
    toggleAlliance,
    clearFilters,
    clear,
  };
}
