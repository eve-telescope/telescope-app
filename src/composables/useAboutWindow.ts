import { WebviewWindow } from '@tauri-apps/api/webviewWindow'

let aboutWindow: WebviewWindow | null = null

export async function openAboutWindow() {
    if (aboutWindow) {
        await aboutWindow.setFocus()
        return
    }

    aboutWindow = new WebviewWindow('about', {
        url: '/about',
        title: 'About Telescope',
        width: 500,
        height: 700,
        resizable: false,
        center: true,
    })

    aboutWindow.once('tauri://destroyed', () => {
        aboutWindow = null
    })
}
