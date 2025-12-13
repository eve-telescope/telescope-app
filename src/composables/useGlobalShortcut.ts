import { ref, onMounted, onUnmounted, watch } from "vue";
import { register, unregister, isRegistered } from "@tauri-apps/plugin-global-shortcut";
import { readText } from "@tauri-apps/plugin-clipboard-manager";
import { info, warn, error as logError } from "@tauri-apps/plugin-log";
import { useSettings } from "./useSettings";

export function useGlobalShortcut(onPaste: (text: string) => void) {
  const { settings, loaded } = useSettings();
  
  const isActive = ref(false);
  const error = ref<string | null>(null);
  const currentShortcut = ref<string | null>(null);

  async function registerShortcut(shortcut: string) {
    if (currentShortcut.value === shortcut && isActive.value) {
      return;
    }

    if (currentShortcut.value) {
      await unregisterShortcut();
    }

    try {
      const alreadyRegistered = await isRegistered(shortcut);
      if (alreadyRegistered) {
        await unregister(shortcut);
      }

      await register(shortcut, async () => {
        if (!settings.value.autoScanOnShortcut) {
          return;
        }
        
        try {
          info(`Global shortcut ${shortcut} triggered`);
          const clipboardText = await readText();
          if (clipboardText && clipboardText.trim()) {
            const lineCount = clipboardText.split("\n").filter(l => l.trim()).length;
            info(`Reading ${lineCount} lines from clipboard`);
            onPaste(clipboardText);
          } else {
            warn("Clipboard is empty or contains only whitespace");
          }
        } catch (e) {
          logError(`Failed to read clipboard: ${e}`);
        }
      });

      currentShortcut.value = shortcut;
      isActive.value = true;
      error.value = null;
      info(`Global shortcut registered: ${shortcut}`);
    } catch (e) {
      logError(`Failed to register global shortcut: ${e}`);
      error.value = String(e);
      isActive.value = false;
    }
  }

  async function unregisterShortcut() {
    if (!currentShortcut.value) return;
    
    try {
      const alreadyRegistered = await isRegistered(currentShortcut.value);
      if (alreadyRegistered) {
        await unregister(currentShortcut.value);
        info(`Global shortcut unregistered: ${currentShortcut.value}`);
      }
      currentShortcut.value = null;
      isActive.value = false;
    } catch (e) {
      logError(`Failed to unregister global shortcut: ${e}`);
    }
  }

  watch(
    () => settings.value.globalShortcut,
    (newShortcut) => {
      if (loaded.value && newShortcut) {
        registerShortcut(newShortcut);
      }
    }
  );

  watch(loaded, (isLoaded) => {
    if (isLoaded && settings.value.globalShortcut) {
      registerShortcut(settings.value.globalShortcut);
    }
  });

  onMounted(() => {
    if (loaded.value && settings.value.globalShortcut) {
      registerShortcut(settings.value.globalShortcut);
    }
  });

  onUnmounted(() => {
    unregisterShortcut();
  });

  return {
    shortcut: currentShortcut,
    displayShortcut: () => settings.value.globalShortcut,
    isActive,
    error,
    updateShortcut: (shortcut: string) => {
      settings.value.globalShortcut = shortcut;
    },
  };
}
