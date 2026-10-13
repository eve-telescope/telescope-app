//! The update check, and the dialog that installs a new release and
//! restarts into it.

use std::path::PathBuf;
use std::pin::pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::Duration;

use futures::future::{Either, select};
use gpui_kit::component::WindowExt as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    App, AppContext as _, AsyncApp, Context, FontWeight, IntoElement, ParentElement as _, Render,
    Styled as _, Task, Window, div, px,
};
use log::{info, warn};
use telescope_core::update::{self, Release};

use crate::runtime;
use crate::theme::{self, CYAN, RED, TEXT_2};
use crate::ui::IconName;
use crate::windows::main_window;

const CHECK_INTERVAL: Duration = Duration::from_secs(60 * 60);
const PROGRESS_TICK: Duration = Duration::from_millis(100);

pub fn init(cx: &mut App) {
    cx.spawn(async move |cx: &mut AsyncApp| {
        loop {
            match runtime::spawn(update::check_for_update()).await {
                Ok(Some(release)) => {
                    cx.update(|cx| show(release, cx));
                    return;
                }
                Ok(None) => {}
                Err(e) => warn!("Update check failed: {}", e),
            }
            cx.background_executor().timer(CHECK_INTERVAL).await;
        }
    })
    .detach();
}

fn show(release: Release, cx: &mut App) {
    let Some(handle) = main_window::handle(cx) else {
        return;
    };
    let _ = handle.update(cx, move |_, window, cx| {
        let view = cx.new(|_| UpdateView::new(release));
        window.open_dialog(cx, move |dialog, _, _| {
            dialog
                .title("Update Available")
                .w(px(440.))
                .child(view.clone())
        });
    });
}

enum Phase {
    Available,
    /// Download progress in percent.
    Installing(Arc<AtomicU8>),
    /// Installed; the new version runs after a restart from this executable.
    Ready(PathBuf),
    Failed(String),
}

struct UpdateView {
    release: Release,
    installable: bool,
    phase: Phase,
    _install: Option<Task<()>>,
}

impl UpdateView {
    fn new(release: Release) -> Self {
        Self {
            installable: release.installable(),
            release,
            phase: Phase::Available,
            _install: None,
        }
    }

    fn install(&mut self, cx: &mut Context<Self>) {
        let progress = Arc::new(AtomicU8::new(0));
        self.phase = Phase::Installing(progress.clone());
        let release = self.release.clone();
        self._install = Some(cx.spawn(async move |this, cx| {
            let mut job = pin!(runtime::spawn(async move {
                update::install(&release, &progress).await
            }));
            let result = loop {
                let tick = pin!(cx.background_executor().timer(PROGRESS_TICK));
                match select(job.as_mut(), tick).await {
                    Either::Left((result, _)) => break result,
                    Either::Right(_) => {
                        let _ = this.update(cx, |_, cx| cx.notify());
                    }
                }
            };
            let _ = this.update(cx, |this, cx| {
                this.phase = match result {
                    Ok(exe) => {
                        info!("Update to v{} installed", this.release.version);
                        Phase::Ready(exe)
                    }
                    Err(e) => {
                        warn!("Update failed: {}", e);
                        Phase::Failed(e)
                    }
                };
                cx.notify();
            });
        }));
        cx.notify();
    }

    fn restart(&mut self, exe: PathBuf, cx: &mut Context<Self>) {
        match update::restart(&exe) {
            Ok(()) => cx.quit(),
            Err(e) => {
                warn!("{}", e);
                self.phase = Phase::Failed(e);
                cx.notify();
            }
        }
    }

    fn message(&self) -> (String, bool) {
        match &self.phase {
            Phase::Available if self.installable => (
                "A new version of Telescope is available. It installs in the background and takes over when Telescope restarts.".into(),
                false,
            ),
            Phase::Available => (
                "A new version of Telescope is available. Download it from the release page.".into(),
                false,
            ),
            Phase::Installing(progress) => (
                format!(
                    "Downloading the update... {}%",
                    progress.load(Ordering::Relaxed)
                ),
                false,
            ),
            Phase::Ready(_) => (
                "The update is installed. Restart Telescope to use it.".into(),
                false,
            ),
            Phase::Failed(e) => (
                format!(
                    "{}. You can download the new version from the release page instead.",
                    e.trim_end_matches('.')
                ),
                true,
            ),
        }
    }
}

impl Render for UpdateView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (message, error) = self.message();
        let url = self.release.url();
        let later = Button::new("update-later")
            .outline()
            .label(if matches!(self.phase, Phase::Installing(_)) {
                "Cancel"
            } else {
                "Later"
            })
            .on_click(|_, window, cx| window.close_dialog(cx));
        let download = Button::new("update-download")
            .primary()
            .icon(IconName::Download)
            .label("Download")
            .on_click(move |_, window, cx| {
                cx.open_url(&url);
                window.close_dialog(cx);
            });

        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme::color(CYAN))
                    .child(format!(
                        "v{} → v{}",
                        env!("CARGO_PKG_VERSION"),
                        self.release.version
                    )),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(theme::color(if error { RED } else { TEXT_2 }))
                    .child(message),
            )
            .child(div().flex().justify_end().gap_2().child(later).map(|el| {
                match &self.phase {
                    Phase::Available if self.installable => el
                        .child({
                            let url = self.release.url();
                            Button::new("update-manual")
                                .ghost()
                                .label("Download manually")
                                .on_click(move |_, _, cx| cx.open_url(&url))
                        })
                        .child(
                            Button::new("update-install")
                                .primary()
                                .icon(IconName::Download)
                                .label("Update")
                                .on_click(cx.listener(|this, _, _, cx| this.install(cx))),
                        ),
                    Phase::Ready(exe) => {
                        let exe = exe.clone();
                        el.child(
                            Button::new("update-restart")
                                .primary()
                                .icon(IconName::RotateCcw)
                                .label("Restart now")
                                .on_click(
                                    cx.listener(move |this, _, _, cx| {
                                        this.restart(exe.clone(), cx)
                                    }),
                                ),
                        )
                    }
                    Phase::Installing(_) => el,
                    Phase::Available | Phase::Failed(_) => el.child(download),
                }
            }))
    }
}
