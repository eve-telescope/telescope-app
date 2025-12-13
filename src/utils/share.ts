const DEEP_LINK_SCHEME = "telescope";
const API_BASE_URL = "https://eve-telescope.com";

export interface ShareResponse {
  code: string;
  url: string;
}

export async function createShare(pilotNames: string): Promise<ShareResponse> {
  const pilots = pilotNames
    .split("\n")
    .map((n) => n.trim())
    .filter((n) => n.length > 0);

  const response = await fetch(`${API_BASE_URL}/api/share`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
    body: JSON.stringify({ pilots }),
  });

  if (!response.ok) {
    throw new Error(`Failed to create share: ${response.statusText}`);
  }

  return response.json();
}

export function createDeepLinkUrl(code: string): string {
  return `${DEEP_LINK_SCHEME}://s/${code}`;
}

export function parseDeepLinkUrl(url: string): string | null {
  try {
    if (!url.startsWith(`${DEEP_LINK_SCHEME}://`)) {
      return null;
    }

    const path = url.replace(`${DEEP_LINK_SCHEME}://`, "");
    const match = path.match(/^s\/([a-zA-Z0-9]+)$/);

    return match ? match[1] : null;
  } catch {
    return null;
  }
}

