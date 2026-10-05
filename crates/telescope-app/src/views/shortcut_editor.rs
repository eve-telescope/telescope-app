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
use crate::theme::{self, CYAN, GREEN, ORANGE, RED, TEXT_1, TEXT_3};
use crate::ui::{Icon, IconName};

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
        .px_1p5()
        .min_w(px(22.))
        .h(px(22.))
        .rounded(px(5.))
        .bg(theme::hairline(0x14))
        .border_b_2()
        .border_color(theme::hairline(0x0a))
        .text_xs()
        .font_weight(FontWeight::SEMIBOLD)
        .flex()
        .items_center()
        .justify_center()
        .text_color(theme::color(if editing { CYAN } else { TEXT_1 }))
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
        let is_default = current == DEFAULT_SHORTCUT;

        div()
            .flex()
            .flex_col()
            .items_end()
            .gap_1()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(
                        div()
                            .id("shortcut-box")
                            .track_focus(&self.focus)
                            .on_key_down(cx.listener(|this, event, _, cx| this.on_key(event, cx)))
                            .flex()
                            .items_center()
                            .gap_1()
                            .h(px(32.))
                            .px_2()
                            .min_w(px(120.))
                            .justify_center()
                            .rounded(px(7.))
                            .border_1()
                            .cursor_pointer()
                            .map(|el| {
                                if self.recording {
                                    el.bg(theme::tint(CYAN, 0x14))
                                        .border_color(theme::tint(CYAN, 0x80))
                                } else {
                                    el.bg(theme::hairline(0x08))
                                        .border_color(theme::hairline(0x14))
                                        .hover(|s| s.border_color(theme::hairline(0x29)))
                                }
                            })
                            .tooltip(|window, cx| {
                                gpui_kit::component::tooltip::Tooltip::new(
                                    "Click to record a new shortcut",
                                )
                                .build(window, cx)
                            })
                            .on_click(cx.listener(|this, _, window, cx| {
                                if !this.recording {
                                    this.start(window, cx)
                                }
                            }))
                            .map(|el| {
                                if self.recording && self.recorded.is_none() {
                                    el.child(
                                        div()
                                            .text_xs()
                                            .text_color(theme::color(CYAN))
                                            .child("Press keys…"),
                                    )
                                } else {
                                    let editing = self.recording;
                                    el.children(
                                        shown
                                            .format_keys(IS_MAC)
                                            .into_iter()
                                            .map(|k| kbd(k, editing)),
                                    )
                                }
                            }),
                    )
                    .map(|el| match recorded {
                        Some(recorded) => el
                            .child(
                                Button::new("confirm-shortcut")
                                    .ghost()
                                    .small()
                                    .icon(
                                        Icon::new(if conflict {
                                            IconName::TriangleAlert
                                        } else {
                                            IconName::Check
                                        })
                                        .text_color(
                                            theme::color(if conflict { ORANGE } else { GREEN }),
                                        ),
                                    )
                                    .tooltip(if conflict {
                                        "Use anyway"
                                    } else {
                                        "Save shortcut"
                                    })
                                    .disabled(!validation.is_some_and(ShortcutValidation::is_valid))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.save(recorded.clone(), cx)
                                    })),
                            )
                            .child(
                                Button::new("cancel-shortcut")
                                    .ghost()
                                    .small()
                                    .icon(Icon::new(IconName::X).text_color(theme::color(TEXT_3)))
                                    .tooltip("Cancel")
                                    .on_click(cx.listener(|this, _, _, cx| this.stop(cx))),
                            ),
                        None if self.recording => el.child(
                            Button::new("cancel-shortcut")
                                .ghost()
                                .small()
                                .icon(Icon::new(IconName::X).text_color(theme::color(TEXT_3)))
                                .tooltip("Cancel (Esc)")
                                .on_click(cx.listener(|this, _, _, cx| this.stop(cx))),
                        ),
                        None => el.when(!is_default, |el| {
                            el.child(
                                Button::new("reset-shortcut")
                                    .ghost()
                                    .small()
                                    .icon(
                                        Icon::new(IconName::RotateCcw)
                                            .text_color(theme::color(TEXT_3)),
                                    )
                                    .tooltip("Reset to default")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.save(DEFAULT_SHORTCUT.to_string(), cx)
                                    })),
                            )
                        }),
                    }),
            )
            .when_some(
                validation
                    .and_then(|v| v.message())
                    .filter(|_| self.recording),
                |el, message| {
                    el.child(
                        div()
                            .text_size(px(10.))
                            .text_color(theme::color(if conflict { ORANGE } else { RED }))
                            .child(message),
                    )
                },
            )
    }
}
