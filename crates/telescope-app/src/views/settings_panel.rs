//! The General settings page: shortcut, account, local data and about.

use std::time::Duration;

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::{Disableable as _, Sizable as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AnyElement, App, AppContext as _, ClipboardItem, Context, Entity, InteractiveElement as _,
    IntoElement, ParentElement as _, Render, SharedString, StatefulInteractiveElement as _,
    Styled as _, Subscription, Task, Window, div, px,
};
use log::{error, info};

use crate::services::Services;
use crate::state::Stores;
use crate::theme::{self, BG_0, GREEN};
use crate::ui::{Icon, IconName, mono};
use crate::views::settings_ui::{group, page, row};
use crate::views::shortcut_editor::ShortcutEditor;
use crate::windows::about;

const FEEDBACK: Duration = Duration::from_secs(2);

pub struct SettingsPanel {
    shortcut: Entity<ShortcutEditor>,
    token: Entity<InputState>,
    show_token_input: bool,
    login_copied: bool,
    cache_cleared: bool,
    feedback: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl SettingsPanel {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let token = cx.new(|cx| InputState::new(window, cx).placeholder("Paste token"));
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
            cache_cleared: false,
            feedback: None,
            _subscriptions: subscriptions,
        }
    }

    fn login_url(cx: &App) -> String {
        format!("{}/eve?desktop=1", Services::get(cx).api_base_url)
    }

    /// Shows a confirmation for a moment, then clears it.
    fn flash(&mut self, cx: &mut Context<Self>, set: fn(&mut Self, bool)) {
        set(self, true);
        self.feedback = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(FEEDBACK).await;
            let _ = this.update(cx, |this, cx| {
                set(this, false);
                cx.notify();
            });
        }));
        cx.notify();
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

    fn account_rows(&self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        if Stores::get(cx).intel.read(cx).is_authenticated() {
            return vec![
                row(
                    "EVE character",
                    Some("Connected. Scans and annotations sync with your intel networks.".into()),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1p5()
                                .text_xs()
                                .text_color(theme::color(GREEN))
                                .child(crate::ui::dot(theme::color(GREEN), 6.))
                                .child("Connected"),
                        )
                        .child(
                            Button::new("logout")
                                .outline()
                                .small()
                                .label("Disconnect")
                                .on_click(|_, _, cx| {
                                    Stores::get(cx)
                                        .intel
                                        .update(cx, |intel, cx| intel.logout(cx))
                                }),
                        ),
                )
                .into_any_element(),
            ];
        }

        let login_url = Self::login_url(cx);
        let token_empty = self.token.read(cx).value().trim().is_empty();
        vec![
            row(
                "EVE character",
                Some("Sign in to share scans and tag pilots with your intel network.".into()),
            )
            .child(
                Button::new("login")
                    .primary()
                    .small()
                    .label("Connect with EVE")
                    .on_click(|_, _, cx| {
                        info!("Opening EVE login in browser");
                        cx.open_url(&Self::login_url(cx))
                    }),
            )
            .into_any_element(),
            row("Login link", Some(SharedString::from(login_url)))
                .font_family(mono())
                .child(
                    Button::new("copy-login")
                        .ghost()
                        .small()
                        .icon(if self.login_copied {
                            Icon::new(IconName::Check).text_color(theme::color(GREEN))
                        } else {
                            Icon::new(IconName::Copy)
                        })
                        .label(if self.login_copied { "Copied" } else { "Copy" })
                        .on_click(cx.listener(|this, _, _, cx| {
                            cx.write_to_clipboard(ClipboardItem::new_string(Self::login_url(cx)));
                            this.flash(cx, |this, on| this.login_copied = on);
                        })),
                )
                .into_any_element(),
            row(
                "Token",
                Some("Paste a token if the browser can't hand it back to Telescope.".into()),
            )
            .child(if self.show_token_input {
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().w(px(220.)).child(Input::new(&self.token).small()))
                    .child(
                        Button::new("submit-token")
                            .primary()
                            .small()
                            .label("Connect")
                            .disabled(token_empty)
                            .on_click(
                                cx.listener(|this, _, window, cx| this.submit_token(window, cx)),
                            ),
                    )
                    .into_any_element()
            } else {
                Button::new("show-token")
                    .ghost()
                    .small()
                    .label("Paste token")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.show_token_input = true;
                        this.token.update(cx, |state, cx| state.focus(window, cx));
                        cx.notify();
                    }))
                    .into_any_element()
            })
            .into_any_element(),
        ]
    }

    fn data_rows(&self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let scan = Stores::get(cx).scan.read(cx);
        let status = scan.sde_status().cloned();
        let syncing = scan.sde_syncing();
        let build = status.as_ref().and_then(|s| s.build_number);
        let latest = status
            .as_ref()
            .and_then(|s| s.latest_build_number)
            .filter(|latest| Some(*latest) != build);
        let error = status.as_ref().and_then(|s| s.last_error.clone());
        let sde_description = match (build, latest, error) {
            (_, _, Some(error)) => format!("Update failed: {error}"),
            (Some(build), Some(latest), None) => {
                format!("Build {build} · build {latest} available")
            }
            (Some(build), None, None) => {
                format!("Build {build}, up to date. Used to classify d-scans.")
            }
            (None, _, None) => "Not downloaded yet. Needed to classify d-scans.".to_string(),
        };

        vec![
            row(
                "Lookup cache",
                Some(
                    "Pilot and killboard data is kept locally so repeat scans are instant.".into(),
                ),
            )
            .child(
                Button::new("clear-cache")
                    .outline()
                    .small()
                    .when(self.cache_cleared, |b| {
                        b.icon(Icon::new(IconName::Check).text_color(theme::color(GREEN)))
                    })
                    .label(if self.cache_cleared {
                        "Cleared"
                    } else {
                        "Clear"
                    })
                    .on_click(cx.listener(
                        |this, _, _, cx| match Services::get(cx).cache.clear() {
                            Ok(()) => {
                                info!("Cache cleared");
                                this.flash(cx, |this, on| this.cache_cleared = on);
                            }
                            Err(e) => error!("Failed to clear cache: {}", e),
                        },
                    )),
            )
            .into_any_element(),
            row("Static data", Some(sde_description.into()))
                .child(
                    Button::new("refresh-sde")
                        .outline()
                        .small()
                        .label(if latest.is_some() {
                            "Update"
                        } else {
                            "Refresh"
                        })
                        .loading(syncing)
                        .disabled(syncing)
                        .on_click(|_, _, cx| {
                            Stores::get(cx)
                                .scan
                                .update(cx, |scan, cx| scan.ensure_sde(cx))
                        }),
                )
                .into_any_element(),
        ]
    }

    fn about_rows(&self) -> Vec<AnyElement> {
        let link = |id: &'static str, label: &'static str, url: &'static str| {
            Button::new(id)
                .ghost()
                .small()
                .label(label)
                .on_click(move |_, _, cx| cx.open_url(url))
        };
        vec![
            row(
                "Telescope",
                Some(format!("Version {}", env!("CARGO_PKG_VERSION")).into()),
            )
            .child(
                Button::new("about")
                    .outline()
                    .small()
                    .label("How to use")
                    .on_click(|_, _, cx| about::open(cx)),
            )
            .into_any_element(),
            row("Links", None)
                .child(
                    div()
                        .flex()
                        .gap_1()
                        .child(link("link-site", "Website", "https://eve-telescope.com"))
                        .child(link(
                            "link-github",
                            "GitHub",
                            "https://github.com/eve-telescope/telescope-app",
                        ))
                        .child(link("link-zkill", "zKillboard", "https://zkillboard.com")),
                )
                .into_any_element(),
        ]
    }
}

impl Render for SettingsPanel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let shortcut_note = if cfg!(target_os = "linux") {
            "Scans the clipboard from anywhere. Needs an X11 session; not available on Wayland."
        } else {
            "Scans the clipboard from anywhere, even while EVE has focus."
        };
        let scanning = vec![
            row("Global shortcut", Some(shortcut_note.into()))
                .child(self.shortcut.clone())
                .into_any_element(),
        ];
        let account = self.account_rows(cx);
        let data = self.data_rows(cx);
        let about = self.about_rows();

        div()
            .id("general-settings")
            .size_full()
            .overflow_y_scroll()
            .bg(theme::color(BG_0))
            .child(
                page("General", "Shortcut, account and local data.")
                    .child(group(Some("Scanning"), scanning))
                    .child(group(Some("Account"), account))
                    .child(group(Some("Data"), data))
                    .child(group(Some("About"), about)),
            )
    }
}
