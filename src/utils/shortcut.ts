// Detect platform
const isMac =
    typeof navigator !== 'undefined' &&
    navigator.platform.toUpperCase().indexOf('MAC') >= 0

// Platform-specific key display
const MAC_KEY_DISPLAY: Record<string, string> = {
    CommandOrControl: '⌘',
    Command: '⌘',
    Control: '⌃',
    Ctrl: '⌃',
    Alt: '⌥',
    Option: '⌥',
    Shift: '⇧',
    Super: '⌘',
    Meta: '⌘',
}

const WIN_KEY_DISPLAY: Record<string, string> = {
    CommandOrControl: 'Ctrl',
    Command: 'Win',
    Control: 'Ctrl',
    Ctrl: 'Ctrl',
    Alt: 'Alt',
    Option: 'Alt',
    Shift: 'Shift',
    Super: 'Win',
    Meta: 'Win',
}

const COMMON_KEY_DISPLAY: Record<string, string> = {
    Enter: '↵',
    Return: '↵',
    Escape: 'Esc',
    Backspace: '⌫',
    Delete: 'Del',
    Tab: 'Tab',
    Space: 'Space',
    ArrowUp: '↑',
    ArrowDown: '↓',
    ArrowLeft: '←',
    ArrowRight: '→',
}

export function formatShortcutKeys(shortcut: string): string[] {
    if (!shortcut) return []

    const keyMap = isMac ? MAC_KEY_DISPLAY : WIN_KEY_DISPLAY

    return shortcut
        .split('+')
        .map(
            (key) => keyMap[key] || COMMON_KEY_DISPLAY[key] || key.toUpperCase()
        )
}

export function formatShortcut(shortcut: string): string {
    return formatShortcutKeys(shortcut).join(isMac ? '' : ' + ')
}

export function formatShortcutVerbose(shortcut: string): string {
    if (!shortcut) return ''

    return shortcut
        .split('+')
        .map((key) => {
            if (key === 'CommandOrControl') return isMac ? 'Cmd' : 'Ctrl'
            if (key === 'Command') return 'Cmd'
            if (key === 'Control') return 'Ctrl'
            return key
        })
        .join(' + ')
}

export function keyEventToShortcut(e: KeyboardEvent): string | null {
    const parts: string[] = []

    if (e.metaKey || e.ctrlKey) {
        parts.push('CommandOrControl')
    }
    if (e.altKey) {
        parts.push('Alt')
    }
    if (e.shiftKey) {
        parts.push('Shift')
    }

    const key = e.key

    if (['Control', 'Meta', 'Alt', 'Shift'].includes(key)) {
        return null
    }

    if (key.length === 1) {
        parts.push(key.toUpperCase())
    } else {
        parts.push(key)
    }

    if (parts.length < 2) {
        return null
    }

    return parts.join('+')
}

export function isValidShortcut(shortcut: string): boolean {
    if (!shortcut) return false

    const parts = shortcut.split('+')

    const hasModifier = parts.some((p) =>
        [
            'CommandOrControl',
            'Command',
            'Control',
            'Alt',
            'Shift',
            'Super',
            'Meta',
        ].includes(p)
    )

    const hasKey = parts.some(
        (p) =>
            ![
                'CommandOrControl',
                'Command',
                'Control',
                'Alt',
                'Shift',
                'Super',
                'Meta',
            ].includes(p)
    )

    return hasModifier && hasKey
}

const SHORTCUT_DESCRIPTIONS: Record<string, string> = {
    'CommandOrControl+C': 'Copy',
    'CommandOrControl+V': 'Paste',
    'CommandOrControl+X': 'Cut',
    'CommandOrControl+Z': 'Undo',
    'CommandOrControl+Y': 'Redo',
    'CommandOrControl+A': 'Select All',
    'CommandOrControl+S': 'Save',
    'CommandOrControl+P': 'Print',
    'CommandOrControl+F': 'Find',
    'CommandOrControl+W': 'Close Window/Tab',
    'CommandOrControl+Q': 'Quit App',
    'CommandOrControl+N': 'New Window/File',
    'CommandOrControl+O': 'Open File',
    'CommandOrControl+T': 'New Tab',
    'CommandOrControl+R': 'Refresh',
    'CommandOrControl+Shift+Z': 'Redo',
    'CommandOrControl+Shift+T': 'Reopen Tab',
    'CommandOrControl+Shift+N': 'New Window (Private)',
    'Alt+Tab': 'Switch Windows',
    'Alt+F4': 'Close Window',
    'CommandOrControl+Tab': 'Switch Tabs',
    'CommandOrControl+Shift+Tab': 'Switch Tabs (Reverse)',
}

export interface ShortcutValidation {
    valid: boolean
    conflict?: string
    message?: string
}

function normalizeShortcut(shortcut: string): string {
    return shortcut
        .split('+')
        .sort((a, b) => {
            const modifiers = [
                'CommandOrControl',
                'Command',
                'Control',
                'Alt',
                'Shift',
                'Super',
                'Meta',
            ]
            const aIdx = modifiers.indexOf(a)
            const bIdx = modifiers.indexOf(b)
            if (aIdx !== -1 && bIdx !== -1) return aIdx - bIdx
            if (aIdx !== -1) return -1
            if (bIdx !== -1) return 1
            return 0
        })
        .join('+')
}

export function validateShortcut(shortcut: string): ShortcutValidation {
    if (!shortcut) {
        return { valid: false, message: 'No shortcut provided' }
    }

    if (!isValidShortcut(shortcut)) {
        return { valid: false, message: 'Must have a modifier key' }
    }

    const normalized = normalizeShortcut(shortcut)
    const conflictAction = SHORTCUT_DESCRIPTIONS[normalized]

    if (conflictAction) {
        return {
            valid: true,
            conflict: conflictAction,
            message: `Usually "${conflictAction}" — will be overridden`,
        }
    }

    return { valid: true }
}

export { isMac }
