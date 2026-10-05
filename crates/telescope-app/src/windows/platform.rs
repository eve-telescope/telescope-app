//! App-wide platform glue: the global scan hotkey and the update check.

use std::time::Duration;

use global_hotkey::hotkey::HotKey;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use gpui_kit::component::WindowExt as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{App, AsyncApp, FontWeight, Global, ParentElement as _, Styled as _, div, px};
use log::{info, warn};
use telescope_core::update::{UpdateInfo, check_for_update};

use crate::runtime;
use crate::state::Stores;
use crate::theme::{self, BG_2, CYAN, TEXT_2};
use crate::ui::IconName;
use crate::windows::main_window;

const HOTKEY_POLL: Duration = Duration::from_millis(50);
const UPDATE_INTERVAL: Duration = Duration::from_secs(60 * 60);

struct Hotkeys {
    manager: GlobalHotKeyManager,
    current: Option<HotKey>,
}

impl Global for Hotkeys {}

pub fn init(cx: &mut App) {
    init_hotkey(cx);
    init_update_check(cx);
}

fn init_hotkey(cx: &mut App) {
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

fn init_update_check(cx: &mut App) {
    cx.spawn(async move |cx: &mut AsyncApp| {
        loop {
            match runtime::spawn(check_for_update()).await {
                Ok(Some(info)) => {
                    cx.update(|cx| show_update(info, cx));
                    return;
                }
                Ok(None) => {}
                Err(e) => warn!("Update check failed: {}", e),
            }
            cx.background_executor().timer(UPDATE_INTERVAL).await;
        }
    })
    .detach();
}

fn show_update(info: UpdateInfo, cx: &mut App) {
    let Some(handle) = main_window::handle(cx) else {
        return;
    };
    let notes: String = if info.release_notes.chars().count() > 300 {
        format!(
            "{}...",
            info.release_notes.chars().take(300).collect::<String>()
        )
    } else {
        info.release_notes.clone()
    };
    let _ = handle.update(cx, move |_, window, cx| {
        window.open_dialog(cx, move |dialog, _, _| {
            let url = info.release_url.clone();
            dialog
                .title("Update Available")
                .w(px(440.))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme::color(CYAN))
                                .child(format!("v{} → v{}", info.current_version, info.latest_version)),
                        )
                        .child(div().text_sm().text_color(theme::color(TEXT_2)).child(
                            "A new version of Telescope is available. Update to get the latest features and bug fixes.",
                        ))
                        .when(!notes.is_empty(), |el| {
                            el.child(
                                div()
                                    .p_3()
                                    .rounded_sm()
                                    .bg(theme::color(BG_2))
                                    .text_xs()
                                    .text_color(theme::color(TEXT_2))
                                    .child(notes.clone()),
                            )
                        })
                        .child(
                            div()
                                .flex()
                                .justify_end()
                                .gap_2()
                                .child(
                                    Button::new("update-later")
                                        .outline()
                                        .label("Later")
                                        .on_click(|_, window, cx| window.close_dialog(cx)),
                                )
                                .child(
                                    Button::new("update-download")
                                        .primary()
                                        .icon(IconName::Download)
                                        .label("Download")
                                        .on_click(move |_, window, cx| {
                                            cx.open_url(&url);
                                            window.close_dialog(cx);
                                        }),
                                ),
                        ),
                )
        });
    });
}
