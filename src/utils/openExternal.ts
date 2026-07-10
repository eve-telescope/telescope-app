import { invoke } from '@tauri-apps/api/core'

/**
 * Opens an http(s) URL in the default browser via the Rust backend, which
 * logs every attempt/failure and strips LD_PRELOAD-style launch workarounds
 * before spawning the opener on Linux. Rejects with a readable message when
 * the browser could not be launched — callers should surface it, not
 * swallow it.
 */
export function openExternalUrl(url: string): Promise<void> {
    return invoke('open_external', { url })
}
