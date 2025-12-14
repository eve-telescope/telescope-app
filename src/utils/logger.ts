import {
    attachConsole,
    info,
    warn,
    error,
    debug,
    trace,
} from '@tauri-apps/plugin-log'

let detachFn: (() => void) | null = null

export async function initLogger() {
    try {
        detachFn = await attachConsole()
        info('Frontend logger initialized')
    } catch (e) {
        console.error('Failed to attach console to Tauri logger:', e)
    }
}

export function detachLogger() {
    if (detachFn) {
        detachFn()
        detachFn = null
    }
}

function forwardConsole(
    fnName: 'log' | 'debug' | 'info' | 'warn' | 'error',
    logger: (message: string) => Promise<void>
) {
    const original = console[fnName]
    console[fnName] = (...args: unknown[]) => {
        original(...args)
        const message = args
            .map((arg) =>
                typeof arg === 'object' ? JSON.stringify(arg) : String(arg)
            )
            .join(' ')
        logger(message).catch(() => {})
    }
}

export function forwardConsoleLogs() {
    forwardConsole('log', trace)
    forwardConsole('debug', debug)
    forwardConsole('info', info)
    forwardConsole('warn', warn)
    forwardConsole('error', error)
}

export const log = {
    trace,
    debug,
    info,
    warn,
    error,
}
