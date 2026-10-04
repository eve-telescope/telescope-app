use std::time::Duration;

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{InputEvent, Textarea, TextareaState};
use gpui_kit::component::{Disableable as _, Sizable as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AppContext as _, Context, Entity, InteractiveElement as _, IntoElement, ParentElement as _,
    Render, SharedString, StatefulInteractiveElement as _, Styled as _, Subscription, Task, Window,
    div, px, relative,
};
use telescope_core::models::{NetworkScan, PilotIntel};
use telescope_core::view::format::relative_time_short;
use telescope_core::view::scan_input::split_pilot_names;

use crate::state::Stores;
use crate::state::intel::IntelEvent;
use crate::theme::{self, BG_1, BG_2, BG_HOVER, BORDER, CYAN, GREEN, TEXT_1, TEXT_3};
use crate::ui::{Icon, IconName, mono, section_title};
use crate::views::share_button::ShareButton;
use crate::views::threat_summary::threat_summary;

const PROGRESS_DELAY: Duration = Duration::from_millis(500);

pub struct InputPanel {
    text: Entity<TextareaState>,
    share: Entity<ShareButton>,
    /// The last non-empty result, so the summary doesn't blank during a rescan.
    summary_pilots: Vec<PilotIntel>,
    show_progress: bool,
    progress_timer: Option<Task<()>>,
    recent_scans: Vec<NetworkScan>,
    recent_network: Option<i64>,
    refreshing_scans: bool,
    scans_task: Option<Task<()>>,
    _clock: Task<()>,
    _subscriptions: Vec<Subscription>,
}

impl InputPanel {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let text = cx
            .new(|cx| TextareaState::new(window, cx).placeholder("Paste local or D-scan text..."));
        let stores = Stores::get(cx);
        let subscriptions = vec![
            cx.subscribe_in(&text, window, |this, _, event, _, cx| match event {
                InputEvent::Change => cx.notify(),
                InputEvent::PressEnter {
                    secondary: true, ..
                } => this.scan(cx),
                _ => {}
            }),
            cx.observe(&stores.scan, |this, scan, cx| {
                let scan = scan.read(cx);
                if !scan.pilots().is_empty() {
                    this.summary_pilots = scan.pilots().to_vec();
                }
                let loading = scan.loading();
                if loading && this.progress_timer.is_none() && !this.show_progress {
                    this.progress_timer = Some(cx.spawn(async move |this, cx| {
                        cx.background_executor().timer(PROGRESS_DELAY).await;
                        let _ = this.update(cx, |this, cx| {
                            this.show_progress = true;
                            cx.notify();
                        });
                    }));
                } else if !loading {
                    this.progress_timer = None;
                    this.show_progress = false;
                }
                cx.notify();
            }),
            cx.observe(&stores.intel, |this, _, cx| this.sync_recent_scans(cx)),
            cx.subscribe(&stores.intel, |this, _, event, cx| {
                if let IntelEvent::ScanShared(scan) = event {
                    this.recent_scans.insert(0, scan.clone());
                    this.recent_scans.truncate(10);
                    cx.notify();
                }
            }),
        ];

        // Relative scan times ("5m") only need to tick every 30 seconds.
        let clock = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_secs(30))
                    .await;
                if this.update(cx, |_, cx| cx.notify()).is_err() {
                    return;
                }
            }
        });

        let share = cx.new(|_| ShareButton::new(text.clone()));
        let mut panel = Self {
            text,
            share,
            summary_pilots: Vec::new(),
            show_progress: false,
            progress_timer: None,
            recent_scans: Vec::new(),
            recent_network: None,
            refreshing_scans: false,
            scans_task: None,
            _clock: clock,
            _subscriptions: subscriptions,
        };
        panel.sync_recent_scans(cx);
        panel
    }

    pub fn set_text(&mut self, text: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.text.update(cx, |state, cx| {
            state.set_value(text.to_string(), window, cx)
        });
        cx.notify();
    }

    fn value(&self, cx: &Context<Self>) -> SharedString {
        self.text.read(cx).value()
    }

    fn scan(&mut self, cx: &mut Context<Self>) {
        let text = self.value(cx);
        let scan = Stores::get(cx).scan;
        if text.trim().is_empty() || scan.read(cx).loading() {
            return;
        }
        scan.update(cx, |scan, cx| scan.submit(&text, cx));
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.text
            .update(cx, |state, cx| state.set_value("", window, cx));
        Stores::get(cx).scan.update(cx, |scan, cx| scan.clear(cx));
        cx.notify();
    }

    fn sync_recent_scans(&mut self, cx: &mut Context<Self>) {
        let intel = Stores::get(cx).intel.read(cx);
        let network = intel
            .active_network_id()
            .filter(|_| intel.is_authenticated());
        if network != self.recent_network {
            self.recent_network = network;
            self.load_recent_scans(cx);
        }
    }

    fn load_recent_scans(&mut self, cx: &mut Context<Self>) {
        let Some(network_id) = self.recent_network else {
            self.recent_scans.clear();
            self.scans_task = None;
            return;
        };
        self.refreshing_scans = true;
        let fetch = Stores::get(cx).intel.read(cx).fetch_scans(network_id, 1);
        self.scans_task = Some(cx.spawn(async move |this, cx| {
            let result = fetch.await;
            let _ = this.update(cx, |this, cx| {
                this.recent_scans = result
                    .map(|page| page.data.into_iter().take(10).collect())
                    .unwrap_or_default();
                this.refreshing_scans = false;
                cx.notify();
            });
        }));
        cx.notify();
    }

    fn render_recent_scans(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let now = chrono::Utc::now();
        div()
            .flex()
            .flex_col()
            .h(px(208.))
            .flex_none()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_3()
                    .pt_3()
                    .pb_1p5()
                    .child(section_title("RECENT SCANS"))
                    .child(
                        Button::new("refresh-scans")
                            .ghost()
                            .xsmall()
                            .icon(IconName::RefreshCw)
                            .loading(self.refreshing_scans)
                            .tooltip("Refresh scans")
                            .on_click(cx.listener(|this, _, _, cx| this.load_recent_scans(cx))),
                    ),
            )
            .child(
                div()
                    .id("recent-scans")
                    .flex_1()
                    .overflow_y_scroll()
                    .children(self.recent_scans.iter().map(|scan| {
                        let raw = scan.raw_text.clone();
                        let lines = raw.lines().filter(|l| !l.trim().is_empty()).count();
                        let dscan = scan.scan_type == "dscan";
                        div()
                            .id(("recent-scan", scan.id as u64))
                            .flex()
                            .items_center()
                            .gap_2()
                            .px_3()
                            .py_1p5()
                            .cursor_pointer()
                            .hover(|s| s.bg(theme::color(BG_HOVER)))
                            .child(
                                Icon::new(if dscan {
                                    IconName::Radar
                                } else {
                                    IconName::Users
                                })
                                .size_3()
                                .text_color(theme::color(if dscan { CYAN } else { GREEN })),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .truncate()
                                    .text_size(px(10.))
                                    .text_color(theme::color(TEXT_1))
                                    .child(
                                        scan.submitted_by
                                            .as_ref()
                                            .map(|s| s.character_name.clone())
                                            .unwrap_or_else(|| "Unknown".into()),
                                    ),
                            )
                            .child(
                                div()
                                    .text_size(px(9.))
                                    .text_color(theme::color(TEXT_3))
                                    .child(lines.to_string()),
                            )
                            .child(
                                div()
                                    .w(px(24.))
                                    .text_right()
                                    .text_size(px(9.))
                                    .text_color(theme::color(TEXT_3))
                                    .child(
                                        relative_time_short(&scan.created_at, now)
                                            .unwrap_or_default(),
                                    ),
                            )
                            .on_click(cx.listener(move |_, _, _, cx| {
                                Stores::get(cx).scan.update(cx, |scan, cx| {
                                    scan.load(&raw, cx);
                                });
                            }))
                    })),
            )
    }
}

impl Render for InputPanel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let stores = Stores::get(cx);
        let scan = stores.scan.read(cx);
        let loading = scan.loading() || scan.dscan_loading();
        let progress = scan.progress().filter(|_| self.show_progress);
        let value = self.value(cx);
        let empty = value.trim().is_empty();
        let pilot_count = split_pilot_names(&value).len();
        let show_recent = {
            let intel = stores.intel.read(cx);
            intel.is_authenticated()
                && intel.active_network().is_some()
                && !self.recent_scans.is_empty()
        };

        div()
            .w(px(256.))
            .h_full()
            .flex()
            .flex_col()
            .flex_none()
            .bg(theme::color(BG_1))
            .border_r_1()
            .border_color(theme::color(BORDER))
            .child(
                div()
                    .p_3()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .min_h_0()
                    .border_b_1()
                    .border_color(theme::color(BORDER))
                    .child(section_title("PASTE LOCAL OR D-SCAN").mb_2())
                    .child(
                        Textarea::new(&self.text)
                            .flex_1()
                            .min_h(px(128.))
                            .font_family(mono())
                            .text_size(px(12.)),
                    )
                    .child(
                        div()
                            .mt_2()
                            .text_size(px(9.))
                            .text_color(theme::color(TEXT_3))
                            .child("Telescope will infer the input type from the pasted message format."),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .mt_2()
                            .child(
                                Button::new("scan")
                                    .primary()
                                    .flex_1()
                                    .loading(loading)
                                    .disabled(loading || empty)
                                    .label(if pilot_count > 0 {
                                        format!("SCAN  {}", pilot_count)
                                    } else {
                                        "SCAN".to_string()
                                    })
                                    .on_click(cx.listener(|this, _, _, cx| this.scan(cx))),
                            )
                            .child(
                                Button::new("clear")
                                    .outline()
                                    .icon(IconName::X)
                                    .disabled(empty)
                                    .tooltip("Clear")
                                    .on_click(cx.listener(|this, _, window, cx| this.clear(window, cx))),
                            ),
                    )
                    .when_some(progress, |el, progress| {
                        let percent = if progress.total == 0 {
                            0.
                        } else {
                            progress.current as f32 / progress.total as f32
                        };
                        el.child(
                            div()
                                .mt_3()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(
                                    div()
                                        .flex()
                                        .justify_between()
                                        .text_size(px(9.))
                                        .text_color(theme::color(TEXT_3))
                                        .child(if progress.cache_hits > 0 {
                                            div()
                                                .text_color(theme::color(GREEN))
                                                .child(format!("{} cached", progress.cache_hits))
                                        } else {
                                            div().child("Fetching...")
                                        })
                                        .child(format!("{}/{}", progress.current, progress.total)),
                                )
                                .child(
                                    div()
                                        .h(px(6.))
                                        .rounded_full()
                                        .bg(theme::color(BG_2))
                                        .overflow_hidden()
                                        .child(
                                            div()
                                                .h_full()
                                                .w(relative(percent))
                                                .bg(theme::color(CYAN)),
                                        ),
                                ),
                        )
                    }),
            )
            .child(
                div()
                    .p_3()
                    .flex_none()
                    .border_b_1()
                    .border_color(theme::color(BORDER))
                    .child(section_title("THREAT SUMMARY").mb_2())
                    .child(if self.summary_pilots.is_empty() {
                        div()
                            .text_size(px(9.))
                            .text_color(theme::color(TEXT_3))
                            .child("No scan results yet")
                            .into_any_element()
                    } else {
                        threat_summary(&self.summary_pilots).into_any_element()
                    }),
            )
            .child(
                div()
                    .p_3()
                    .flex_none()
                    .border_b_1()
                    .border_color(theme::color(BORDER))
                    .child(self.share.clone()),
            )
            .when(show_recent, |el| el.child(self.render_recent_scans(cx)))
    }
}
