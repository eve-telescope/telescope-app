// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // WebKitGTK's DMABUF renderer is broken on a range of Linux setups
    // (blank/white window, crash on resize, "AcceleratedSurfaceDMABuf was
    // unable to construct a complete framebuffer") — see tauri-apps/tauri#9394
    // and https://v2.tauri.app/learn/linux-graphics/. Telescope has no
    // WebGL/canvas-heavy views, so losing the faster rendering path costs
    // little compared to users needing shell workarounds to launch at all.
    // An explicitly set value always wins (WEBKIT_DISABLE_DMABUF_RENDERER=0
    // re-enables the fast path on known-good setups).
    #[cfg(target_os = "linux")]
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    telescope_lib::run()
}
