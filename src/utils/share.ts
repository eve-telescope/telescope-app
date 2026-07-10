import { API_BASE_URL } from './config'

export interface ShareResponse {
    code: string
    url: string
}

export async function createShare(pilotNames: string): Promise<ShareResponse> {
    const pilots = pilotNames
        .split('\n')
        .map((n) => n.trim())
        .filter((n) => n.length > 0)

    const response = await fetch(`${API_BASE_URL}/api/share`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            Accept: 'application/json',
        },
        body: JSON.stringify({ pilots }),
    })

    if (!response.ok) {
        throw new Error(`Failed to create share: ${response.statusText}`)
    }

    return response.json()
}

// Deep link parsing/dispatch lives in Rust (src-tauri/src/deep_link.rs) so
// auth tokens never transit the frontend; see useDeepLink for the share flow.
