import { API_BASE_URL } from './config'

const DEEP_LINK_SCHEME = 'telescope'

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

export function createDeepLinkUrl(code: string): string {
    return `${DEEP_LINK_SCHEME}://s/${code}`
}

export type DeepLinkResult =
    | { type: 'share'; code: string }
    | { type: 'auth'; token: string }
    | null

export function parseDeepLinkUrl(url: string): DeepLinkResult {
    try {
        if (!url.startsWith(`${DEEP_LINK_SCHEME}://`)) {
            return null
        }

        const path = url.replace(`${DEEP_LINK_SCHEME}://`, '')

        // telescope://auth?token=XXX
        const authMatch = path.match(/^auth\?token=(.+)$/)
        if (authMatch) {
            return { type: 'auth', token: decodeURIComponent(authMatch[1]) }
        }

        // telescope://s/{code}
        const shareMatch = path.match(/^s\/([a-zA-Z0-9]+)$/)
        if (shareMatch) {
            return { type: 'share', code: shareMatch[1] }
        }

        return null
    } catch {
        return null
    }
}
