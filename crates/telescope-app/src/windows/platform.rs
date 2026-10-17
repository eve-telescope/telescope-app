//! App-wide platform glue: the global scan and period hotkeys.

use std::time::Duration;

use global_hotkey::hotkey::HotKey;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use gpui_kit::{App, AsyncApp, Global};
use log::{info, warn};

use crate::state::Stores;
use crate::views::shortcut_editor::ShortcutSlot;
use crate::windows::main_window;

const HOTKEY_POLL: Duration = Duration::from_millis(50);

struct Hotkeys {
    manager: GlobalHotKeyManager,
    scan: Option<HotKey>,
    period: Option<HotKey>,
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
        scan: None,
        period: None,
    });
    apply_shortcuts(cx);

    let settings = Stores::get(cx).settings;
    cx.observe(&settings, |_, cx| apply_shortcuts(cx)).detach();

    // The hotkey crate delivers presses on its own channel; poll it from the
    // UI thread so the scan can update entities directly.
    cx.spawn(async move |cx: &mut AsyncApp| {
        loop {
            cx.background_executor().timer(HOTKEY_POLL).await;
            while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                if event.state() == HotKeyState::Pressed {
                    cx.update(|cx| on_press(event.id(), cx));
                }
            }
        }
    })
    .detach();
}

fn apply_shortcuts(cx: &mut App) {
    apply_shortcut(ShortcutSlot::Scan, cx);
    apply_shortcut(ShortcutSlot::Period, cx);
}

/// Registers the shortcut for `slot` from settings, replacing the previous
/// one. An empty shortcut just unregisters it.
fn apply_shortcut(slot: ShortcutSlot, cx: &mut App) {
    let wanted = slot
        .get(Stores::get(cx).settings.read(cx).get())
        .to_string();
    if !cx.has_global::<Hotkeys>() {
        return;
    }
    let parsed = wanted.parse::<HotKey>().ok();
    let Hotkeys {
        manager,
        scan,
        period,
    } = cx.global_mut::<Hotkeys>();
    let current = match slot {
        ShortcutSlot::Scan => scan,
        ShortcutSlot::Period => period,
    };
    if parsed == *current {
        return;
    }
    if let Some(old) = current.take() {
        let _ = manager.unregister(old);
    }
    match parsed {
        Some(hotkey) => match manager.register(hotkey) {
            Ok(()) => {
                info!("Registered global shortcut {}", wanted);
                *current = Some(hotkey);
            }
            Err(e) => warn!("Failed to register global shortcut {}: {}", wanted, e),
        },
        None if wanted.is_empty() => {}
        None => warn!("Invalid global shortcut {:?}", wanted),
    }
}

fn on_press(id: u32, cx: &mut App) {
    let Some(hotkeys) = cx.try_global::<Hotkeys>() else {
        return;
    };
    if hotkeys.scan.is_some_and(|hotkey| hotkey.id() == id) {
        scan_clipboard(cx);
    } else if hotkeys.period.is_some_and(|hotkey| hotkey.id() == id) {
        Stores::get(cx)
            .settings
            .update(cx, |settings, cx| settings.cycle_period(cx));
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
