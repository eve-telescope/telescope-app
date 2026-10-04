use std::time::Duration;

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::{Disableable as _, Sizable as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AppContext as _, ClipboardItem, Context, Div, Entity, FontWeight, InteractiveElement as _,
    IntoElement, ParentElement as _, Render, StatefulInteractiveElement as _, Styled as _,
    Subscription, Task, Window, div, px,
};
use log::{error, info};

use crate::services::Services;
use crate::state::Stores;
use crate::theme::{self, BG_0, BG_1, BORDER, CYAN, GREEN, TEXT_1, TEXT_2, TEXT_3};
use crate::ui::{Icon, IconName, mono, section_title};
use crate::views::shortcut_editor::ShortcutEditor;
use crate::windows::about;

pub struct SettingsPanel {
    shortcut: Entity<ShortcutEditor>,
    token: Entity<InputState>,
    show_token_input: bool,
    login_copied: bool,
    copy_reset: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl SettingsPanel {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let token = cx.new(|cx| InputState::new(window, cx).placeholder("Paste token..."));
        let stores = Stores::get(cx);
        let subscriptions = vec![
            cx.subscribe_in(&token, window, |this, _, event, window, cx| match event {
                InputEvent::PressEnter { .. } => this.submit_token(window, cx),
                InputEvent::Change => cx.notify(),
                _ => {}
            }),
            cx.observe(&stores.intel, |_, _, cx| cx.notify()),
            cx.observe(&stores.scan, |_, _, cx| cx.notify()),
            cx.observe(&stores.settings, |_, _, cx| cx.notify()),
        ];
        Self {
            shortcut: cx.new(|cx| ShortcutEditor::new(window, cx)),
            token,
            show_token_input: false,
            login_copied: false,
            copy_reset: None,
            _subscriptions: subscriptions,
        }
    }

    fn login_url(cx: &gpui_kit::App) -> String {
        format!("{}/eve?desktop=1", Services::get(cx).api_base_url)
    }

    fn submit_token(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let token = self.token.read(cx).value().trim().to_string();
        if token.is_empty() {
            return;
        }
        self.token
            .update(cx, |state, cx| state.set_value("", window, cx));
        self.show_token_input = false;
        Stores::get(cx)
            .intel
            .update(cx, |intel, cx| intel.set_api_token(token, cx));
        cx.notify();
    }

    fn copy_login_url(&mut self, cx: &mut Context<Self>) {
        cx.write_to_clipboard(ClipboardItem::new_string(Self::login_url(cx)));
        self.login_copied = true;
        self.copy_reset = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_secs(2)).await;
            let _ = this.update(cx, |this, cx| {
                this.login_copied = false;
                cx.notify();
            });
        }));
        cx.notify();
    }

    fn render_account(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let authenticated = Stores::get(cx).intel.read(cx).is_authenticated();
        let login_url = Self::login_url(cx);
        section(
            "INTEL ACCOUNT",
            "Connect or disconnect your character for network access.",
        )
        .child(if authenticated {
            div()
                .mt_4()
                .child(
                    Button::new("logout")
                        .outline()
                        .label("Disconnect Character")
                        .on_click(|_, _, cx| {
                            Stores::get(cx)
                                .intel
                                .update(cx, |intel, cx| intel.logout(cx))
                        }),
                )
                .into_any_element()
        } else {
            div()
                .mt_4()
                .w_full()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    Button::new("login")
                        .outline()
                        .w_full()
                        .label("Connect with EVE")
                        .on_click(move |_, _, cx| {
                            info!("Opening EVE login in browser");
                            cx.open_url(&format!(
                                "{}/eve?desktop=1",
                                Services::get(cx).api_base_url
                            ))
                        }),
                )
                .child(
                    div()
                        .rounded_sm()
                        .border_1()
                        .border_color(theme::color(BORDER))
                        .bg(theme::color(BG_1))
                        .px_2()
                        .py_1p5()
                        .child(
                            div()
                                .mb_1()
                                .text_size(px(10.))
                                .text_color(theme::color(TEXT_3))
                                .child("Or open this link in your browser:"),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1p5()
                                .child(
                                    div()
                                        .flex_1()
                                        .truncate()
                                        .font_family(mono())
                                        .text_size(px(10.))
                                        .text_color(theme::color(TEXT_2))
                                        .child(login_url),
                                )
                                .child(
                                    Button::new("copy-login")
                                        .ghost()
                                        .xsmall()
                                        .icon(if self.login_copied {
                                            Icon::new(IconName::Check)
                                                .text_color(theme::color(GREEN))
                                        } else {
                                            Icon::new(IconName::Copy)
                                        })
                                        .tooltip(if self.login_copied {
                                            "Copied!"
                                        } else {
                                            "Copy link"
                                        })
                                        .on_click(
                                            cx.listener(|this, _, _, cx| this.copy_login_url(cx)),
                                        ),
                                ),
                        ),
                )
                .child(if self.show_token_input {
                    let empty = self.token.read(cx).value().trim().is_empty();
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(Input::new(&self.token).small().font_family(mono()))
                        .child(
                            Button::new("submit-token")
                                .primary()
                                .w_full()
                                .label("Connect")
                                .disabled(empty)
                                .on_click(
                                    cx.listener(|this, _, window, cx| {
                                        this.submit_token(window, cx)
                                    }),
                                ),
                        )
                        .into_any_element()
                } else {
                    Button::new("show-token")
                        .ghost()
                        .small()
                        .w_full()
                        .label("Paste token manually")
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.show_token_input = true;
                            this.token.update(cx, |state, cx| state.focus(window, cx));
                            cx.notify();
                        }))
                        .into_any_element()
                })
                .into_any_element()
        })
    }

    fn render_sde(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let scan = Stores::get(cx).scan.read(cx);
        let status = scan.sde_status().cloned();
        let syncing = scan.sde_syncing();
        let build = status.as_ref().and_then(|s| s.build_number);
        let latest = status
            .as_ref()
            .and_then(|s| s.latest_build_number)
            .filter(|latest| Some(*latest) != build);
        let last_error = status.and_then(|s| s.last_error);

        div()
            .px_5()
            .py_4()
            .child(
                div()
                    .flex()
                    .items_start()
                    .justify_between()
                    .gap_3()
                    .child(section(
                        "SDE CACHE",
                        "Keep the D-scan classification data synced with the latest EVE static export.",
                    ))
                    .child(
                        div()
                            .flex()
                            .flex_none()
                            .items_center()
                            .gap_1()
                            .px_2()
                            .py_0p5()
                            .rounded_sm()
                            .border_1()
                            .border_color(theme::color(BORDER))
                            .text_xs()
                            .child(Icon::new(IconName::HardDrive).size_3())
                            .child(match build {
                                Some(b) => format!("Build {b}"),
                                None => "Not cached".into(),
                            }),
                    ),
            )
            .when_some(latest, |el, latest| {
                el.child(
                    div()
                        .mt_2()
                        .text_xs()
                        .text_color(theme::color(CYAN))
                        .child(format!("Latest available build: {latest}")),
                )
            })
            .when_some(last_error, |el, e| {
                el.child(div().mt_2().text_xs().text_color(theme::color(crate::theme::RED)).child(e))
            })
            .child(
                Button::new("refresh-sde")
                    .outline()
                    .mt_4()
                    .icon(IconName::RefreshCw)
                    .label("Refresh SDE Cache")
                    .loading(syncing)
                    .disabled(syncing)
                    .on_click(|_, _, cx| {
                        Stores::get(cx).scan.update(cx, |scan, cx| scan.ensure_sde(cx))
                    }),
            )
    }
}

fn section(title: &'static str, description: &'static str) -> Div {
    div()
        .flex()
        .flex_col()
        .items_start()
        .child(section_title(title))
        .child(
            div()
                .mt_2()
                .text_xs()
                .text_color(theme::color(TEXT_2))
                .child(description),
        )
}

impl Render for SettingsPanel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cell = || {
            div()
                .flex_1()
                .min_w_0()
                .border_b_1()
                .border_color(theme::color(BORDER))
        };
        div()
            .id("settings")
            .size_full()
            .overflow_y_scroll()
            .bg(theme::color(BG_0))
            .child(
                div()
                    .px_5()
                    .py_4()
                    .border_b_1()
                    .border_color(theme::color(BORDER))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme::color(TEXT_1))
                            .child("GENERAL SETTINGS"),
                    )
                    .child(
                        div()
                            .mt_1()
                            .text_xs()
                            .text_color(theme::color(TEXT_3))
                            .child("Configure shortcut behavior and utility actions for Telescope."),
                    ),
            )
            .child(
                div()
                    .px_5()
                    .py_4()
                    .border_b_1()
                    .border_color(theme::color(BORDER))
                    .child(self.shortcut.clone()),
            )
            .child(
                div()
                    .flex()
                    .child(
                        cell()
                            .border_r_1()
                            .px_5()
                            .py_4()
                            .child(self.render_account(cx)),
                    )
                    .child(
                        cell().px_5().py_4().child(
                            section("CACHE", "Clear locally cached data if you need to force a refresh.")
                                .child(
                                    Button::new("clear-cache")
                                        .outline()
                                        .mt_4()
                                        .icon(IconName::Trash)
                                        .label("Clear Cache")
                                        .on_click(|_, _, cx| {
                                            match Services::get(cx).cache.clear() {
                                                Ok(()) => info!("Cache cleared"),
                                                Err(e) => error!("Failed to clear cache: {}", e),
                                            }
                                        }),
                                ),
                        ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .child(cell().border_r_1().child(self.render_sde(cx)))
                    .child(
                        cell().px_5().py_4().child(
                            section(
                                "HELP",
                                "Open the about window for version info, usage details, and support links.",
                            )
                            .child(
                                Button::new("about")
                                    .outline()
                                    .mt_4()
                                    .icon(IconName::Info)
                                    .label("About Telescope")
                                    .on_click(|_, _, cx| about::open(cx)),
                            ),
                        ),
                    ),
            )
    }
}
