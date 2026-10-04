use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{Disableable as _, Sizable as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    Context, FocusHandle, Focusable, FontWeight, InteractiveElement as _, IntoElement,
    KeyDownEvent, ParentElement as _, Render, StatefulInteractiveElement as _, Styled as _,
    Subscription, Window, div, px,
};
use telescope_core::settings::DEFAULT_SHORTCUT;
use telescope_core::view::shortcut::{Modifiers, Shortcut, ShortcutValidation};

use crate::state::Stores;
use crate::theme::{self, BG_2, BG_3, BORDER, CYAN, GREEN, ORANGE, RED, TEXT_3};
use crate::ui::{Icon, IconName, section_title};

const IS_MAC: bool = cfg!(target_os = "macos");

/// Maps GPUI key names to the DOM `KeyboardEvent.key` names the shortcut
/// model uses.
fn dom_key(key: &str) -> String {
    match key {
        "enter" => "Enter".into(),
        "escape" => "Escape".into(),
        "backspace" => "Backspace".into(),
        "delete" => "Delete".into(),
        "tab" => "Tab".into(),
        "space" => "Space".into(),
        "up" => "ArrowUp".into(),
        "down" => "ArrowDown".into(),
        "left" => "ArrowLeft".into(),
        "right" => "ArrowRight".into(),
        "home" => "Home".into(),
        "end" => "End".into(),
        "pageup" => "PageUp".into(),
        "pagedown" => "PageDown".into(),
        other if other.starts_with('f') && other[1..].parse::<u8>().is_ok() => other.to_uppercase(),
        other => other.to_string(),
    }
}

pub struct ShortcutEditor {
    focus: FocusHandle,
    recording: bool,
    recorded: Option<Shortcut>,
    _blur: Subscription,
}

impl ShortcutEditor {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus = cx.focus_handle();
        let blur = cx.on_blur(&focus, window, |this, _, cx| {
            if this.recording && this.recorded.is_none() {
                this.stop(cx);
            }
        });
        Self {
            focus,
            recording: false,
            recorded: None,
            _blur: blur,
        }
    }

    fn start(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.recording = true;
        self.recorded = None;
        window.focus(&self.focus, cx);
        cx.notify();
    }

    fn stop(&mut self, cx: &mut Context<Self>) {
        self.recording = false;
        self.recorded = None;
        cx.notify();
    }

    fn save(&mut self, shortcut: String, cx: &mut Context<Self>) {
        Stores::get(cx).settings.update(cx, |settings, cx| {
            settings.update(cx, |s| s.global_shortcut = shortcut)
        });
        self.stop(cx);
    }

    fn on_key(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        if !self.recording {
            return;
        }
        cx.stop_propagation();
        let keystroke = &event.keystroke;
        if keystroke.key == "escape" && !keystroke.modifiers.modified() {
            self.stop(cx);
            return;
        }
        let modifiers = Modifiers {
            cmd_or_ctrl: keystroke.modifiers.platform || keystroke.modifiers.control,
            alt: keystroke.modifiers.alt,
            shift: keystroke.modifiers.shift,
        };
        if let Some(shortcut) = Shortcut::from_key_event(modifiers, &dom_key(&keystroke.key)) {
            self.recorded = Some(shortcut);
            cx.notify();
        }
    }
}

impl Focusable for ShortcutEditor {
    fn focus_handle(&self, _: &gpui_kit::App) -> FocusHandle {
        self.focus.clone()
    }
}

fn kbd(key: String, editing: bool) -> impl IntoElement {
    div()
        .px_2()
        .py_1()
        .min_w(px(28.))
        .rounded_sm()
        .bg(theme::color(BG_3))
        .text_sm()
        .font_weight(FontWeight::MEDIUM)
        .flex()
        .justify_center()
        .when(editing, |el| {
            el.border_1()
                .border_color(theme::tint(CYAN, 0x4d))
                .text_color(theme::color(CYAN))
        })
        .child(key)
}

impl Render for ShortcutEditor {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let current = Stores::get(cx)
            .settings
            .read(cx)
            .get()
            .global_shortcut
            .clone();
        let shown = self
            .recorded
            .clone()
            .or_else(|| Shortcut::parse(&current))
            .unwrap_or_default();
        let validation = self.recorded.as_ref().map(Shortcut::validate);
        let conflict = matches!(validation, Some(ShortcutValidation::Conflict(_)));
        let recorded = self.recorded.as_ref().map(ToString::to_string);

        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(section_title("GLOBAL SHORTCUT"))
                    .when(!self.recording, |el| {
                        el.child(
                            Button::new("reset-shortcut")
                                .ghost()
                                .xsmall()
                                .icon(IconName::RotateCcw)
                                .tooltip("Reset to default")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.save(DEFAULT_SHORTCUT.to_string(), cx)
                                })),
                        )
                    }),
            )
            .child(
                div()
                    .id("shortcut-box")
                    .track_focus(&self.focus)
                    .on_key_down(cx.listener(|this, event, _, cx| this.on_key(event, cx)))
                    .flex()
                    .items_center()
                    .gap_3()
                    .px_3()
                    .py_2p5()
                    .rounded_sm()
                    .border_1()
                    .cursor_pointer()
                    .map(|el| {
                        if self.recording {
                            el.bg(theme::tint(CYAN, 0x1a)).border_color(theme::color(CYAN))
                        } else {
                            el.bg(theme::color(BG_2))
                                .border_color(theme::color(BORDER))
                                .hover(|s| s.border_color(theme::color(TEXT_3)))
                        }
                    })
                    .on_click(cx.listener(|this, _, window, cx| {
                        if !this.recording {
                            this.start(window, cx)
                        }
                    }))
                    .child(
                        Icon::new(IconName::Keyboard)
                            .size_4()
                            .text_color(theme::color(if self.recording { CYAN } else { TEXT_3 })),
                    )
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .map(|el| {
                                if self.recording && self.recorded.is_none() {
                                    el.child(div().text_sm().text_color(theme::color(CYAN)).child("Press keys..."))
                                } else {
                                    let editing = self.recording;
                                    el.children(shown.format_keys(IS_MAC).into_iter().map(|k| kbd(k, editing)))
                                }
                            }),
                    )
                    .when_some(recorded, |el, recorded| {
                        el.child(
                            Button::new("confirm-shortcut")
                                .ghost()
                                .small()
                                .icon(
                                    Icon::new(if conflict { IconName::TriangleAlert } else { IconName::Check })
                                        .text_color(theme::color(if conflict { ORANGE } else { GREEN })),
                                )
                                .tooltip(if conflict { "Use anyway" } else { "Confirm" })
                                .disabled(!validation.is_some_and(ShortcutValidation::is_valid))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    cx.stop_propagation();
                                    this.save(recorded.clone(), cx)
                                })),
                        )
                        .child(
                            Button::new("cancel-shortcut")
                                .ghost()
                                .small()
                                .icon(Icon::new(IconName::X).text_color(theme::color(RED)))
                                .tooltip("Cancel")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    cx.stop_propagation();
                                    this.stop(cx)
                                })),
                        )
                    }),
            )
            .when_some(
                validation.and_then(|v| v.message()).filter(|_| self.recording),
                |el, message| {
                    el.child(
                        div()
                            .text_size(px(10.))
                            .text_color(theme::color(if conflict { ORANGE } else { RED }))
                            .child(message),
                    )
                },
            )
            .child(
                div()
                    .text_size(px(9.))
                    .text_color(theme::color(TEXT_3))
                    .opacity(0.7)
                    .child(if self.recording { "Press Escape to cancel" } else { "Click to change" }),
            )
            .when(cfg!(target_os = "linux"), |el| {
                el.child(
                    div()
                        .text_size(px(9.))
                        .text_color(theme::color(TEXT_3))
                        .child("Global shortcuts need an X11 session; they are not available on Wayland."),
                )
            })
    }
}
