import { WebviewWindow } from '@tauri-apps/api/webviewWindow'

let networkWindow: WebviewWindow | null = null

export async function openNetworkWindow() {
    if (networkWindow) {
        await networkWindow.setFocus()
        return
    }

    networkWindow = new WebviewWindow('networks', {
        url: '/networks',
        title: 'Intel Networks',
        width: 600,
        height: 700,
        resizable: true,
        center: true,
    })

    networkWindow.once('tauri://destroyed', () => {
        networkWindow = null
    })
}
