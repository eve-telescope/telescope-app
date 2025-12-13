// Detect platform
const isMac = typeof navigator !== 'undefined' && navigator.platform.toUpperCase().indexOf('MAC') >= 0;

// Platform-specific key display
const MAC_KEY_DISPLAY: Record<string, string> = {
  "CommandOrControl": "⌘",
  "Command": "⌘",
  "Control": "⌃",
  "Ctrl": "⌃",
  "Alt": "⌥",
  "Option": "⌥",
  "Shift": "⇧",
  "Super": "⌘",
  "Meta": "⌘",
};

const WIN_KEY_DISPLAY: Record<string, string> = {
  "CommandOrControl": "Ctrl",
  "Command": "Win",
  "Control": "Ctrl",
  "Ctrl": "Ctrl",
  "Alt": "Alt",
  "Option": "Alt",
  "Shift": "Shift",
  "Super": "Win",
  "Meta": "Win",
};

const COMMON_KEY_DISPLAY: Record<string, string> = {
  "Enter": "↵",
  "Return": "↵",
  "Escape": "Esc",
  "Backspace": "⌫",
  "Delete": "Del",
  "Tab": "Tab",
  "Space": "Space",
  "ArrowUp": "↑",
  "ArrowDown": "↓",
  "ArrowLeft": "←",
  "ArrowRight": "→",
};

export function formatShortcutKeys(shortcut: string): string[] {
  if (!shortcut) return [];
  
  const keyMap = isMac ? MAC_KEY_DISPLAY : WIN_KEY_DISPLAY;
  
  return shortcut
    .split("+")
    .map(key => keyMap[key] || COMMON_KEY_DISPLAY[key] || key.toUpperCase());
}

export function formatShortcut(shortcut: string): string {
  return formatShortcutKeys(shortcut).join(isMac ? "" : " + ");
}

export function formatShortcutVerbose(shortcut: string): string {
  if (!shortcut) return "";
  
  return shortcut
    .split("+")
    .map(key => {
      if (key === "CommandOrControl") return isMac ? "Cmd" : "Ctrl";
      if (key === "Command") return "Cmd";
      if (key === "Control") return "Ctrl";
      return key;
    })
    .join(" + ");
}

export function keyEventToShortcut(e: KeyboardEvent): string | null {
  const parts: string[] = [];
  
  if (e.metaKey || e.ctrlKey) {
    parts.push("CommandOrControl");
  }
  if (e.altKey) {
    parts.push("Alt");
  }
  if (e.shiftKey) {
    parts.push("Shift");
  }
  
  const key = e.key;
  
  if (["Control", "Meta", "Alt", "Shift"].includes(key)) {
    return null;
  }
  
  if (key.length === 1) {
    parts.push(key.toUpperCase());
  } else {
    parts.push(key);
  }
  
  if (parts.length < 2) {
    return null;
  }
  
  return parts.join("+");
}

export function isValidShortcut(shortcut: string): boolean {
  if (!shortcut) return false;
  
  const parts = shortcut.split("+");
  
  const hasModifier = parts.some(p => 
    ["CommandOrControl", "Command", "Control", "Alt", "Shift", "Super", "Meta"].includes(p)
  );
  
  const hasKey = parts.some(p => 
    !["CommandOrControl", "Command", "Control", "Alt", "Shift", "Super", "Meta"].includes(p)
  );
  
  return hasModifier && hasKey;
}

export { isMac };
