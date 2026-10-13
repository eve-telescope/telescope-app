//! App-wide platform glue: the global scan hotkey.

use std::time::Duration;

use global_hotkey::hotkey::HotKey;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use gpui_kit::{App, AsyncApp, Global};
use log::{info, warn};

use crate::state::Stores;
use crate::windows::main_window;

const HOTKEY_POLL: Duration = Duration::from_millis(50);

struct Hotkeys {
    manager: GlobalHotKeyManager,
    current: Option<HotKey>,
}

impl Global for Hotkeys {}

pub fn init(cx: &mut App) {
    let manager = match GlobalHotKeyManager::new() {
        Ok(manager) => manager,
        Err(e) => {
            warn!("Global hotkeys unavailable: {}", e);
            return;
        }
    };
    cx.set_global(Hotkeys {
        manager,
        current: None,
    });
    apply_shortcut(cx);

    let settings = Stores::get(cx).settings;
    cx.observe(&settings, |_, cx| apply_shortcut(cx)).detach();

    // The hotkey crate delivers presses on its own channel; poll it from the
    // UI thread so the scan can update entities directly.
    cx.spawn(async move |cx: &mut AsyncApp| {
        loop {
            cx.background_executor().timer(HOTKEY_POLL).await;
            while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                if event.state() == HotKeyState::Pressed {
                    cx.update(scan_clipboard);
                }
            }
        }
    })
    .detach();
}

/// Registers the shortcut from settings, replacing the previous one.
fn apply_shortcut(cx: &mut App) {
    let wanted = Stores::get(cx)
        .settings
        .read(cx)
        .get()
        .global_shortcut
        .clone();
    let Some(hotkeys) = cx.try_global::<Hotkeys>() else {
        return;
    };
    let parsed = wanted.parse::<HotKey>().ok();
    if parsed == hotkeys.current {
        return;
    }
    let hotkeys = cx.global_mut::<Hotkeys>();
    if let Some(old) = hotkeys.current.take() {
        let _ = hotkeys.manager.unregister(old);
    }
    match parsed {
        Some(hotkey) => match hotkeys.manager.register(hotkey) {
            Ok(()) => {
                info!("Registered global shortcut {}", wanted);
                hotkeys.current = Some(hotkey);
            }
            Err(e) => warn!("Failed to register global shortcut {}: {}", wanted, e),
        },
        None => warn!("Invalid global shortcut {:?}", wanted),
    }
}

fn scan_clipboard(cx: &mut App) {
    let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) else {
        return;
    };
    if text.trim().is_empty() {
        return;
    }
    info!("Global shortcut pressed, scanning clipboard");
    main_window::scan(&text, cx);
}
