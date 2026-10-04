use std::time::Duration;

use gpui_kit::component::input::TextareaState;
use gpui_kit::{
    ClipboardItem, Context, Entity, FontWeight, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, StatefulInteractiveElement as _, Styled as _, Task, Window, div,
    prelude::FluentBuilder as _, px,
};
use log::error;
use telescope_core::share::create_share;

use crate::runtime;
use crate::services::Services;
use crate::state::Stores;
use crate::theme::{self, BG_2, BORDER, CYAN, CYAN_DIM, GREEN, RED, TEXT_2};
use crate::ui::{Icon, IconName};

#[derive(Clone, Copy, PartialEq)]
enum ShareState {
    Idle,
    Loading,
    Copied,
    Failed,
}

pub struct ShareButton {
    text: Entity<TextareaState>,
    state: ShareState,
    task: Option<Task<()>>,
}

impl ShareButton {
    pub fn new(text: Entity<TextareaState>) -> Self {
        Self {
            text,
            state: ShareState::Idle,
            task: None,
        }
    }

    fn share(&mut self, cx: &mut Context<Self>) {
        if self.state == ShareState::Loading {
            return;
        }
        self.state = ShareState::Loading;
        cx.notify();

        let names = self.text.read(cx).value().to_string();
        let base_url = Services::get(cx).api_base_url.clone();
        let create = runtime::spawn(async move { create_share(&base_url, &names).await });
        self.task = Some(cx.spawn(async move |this, cx| {
            let result = create.await;
            let (state, reset_after) = match result {
                Ok(share) => {
                    cx.update(|cx| cx.write_to_clipboard(ClipboardItem::new_string(share.url)));
                    (ShareState::Copied, Duration::from_secs(2))
                }
                Err(e) => {
                    error!("Failed to create share: {}", e);
                    (ShareState::Failed, Duration::from_secs(3))
                }
            };
            let _ = this.update(cx, |this, cx| {
                this.state = state;
                cx.notify();
            });
            cx.background_executor().timer(reset_after).await;
            let _ = this.update(cx, |this, cx| {
                this.state = ShareState::Idle;
                cx.notify();
            });
        }));
    }
}

impl Render for ShareButton {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let has_results = !Stores::get(cx).scan.read(cx).pilots().is_empty();
        let disabled = self.state == ShareState::Loading || !has_results;
        let (icon, label, accent) = match self.state {
            ShareState::Idle => (IconName::Share2, "SHARE SCAN", None),
            ShareState::Loading => (IconName::LoaderCircle, "CREATING LINK...", None),
            ShareState::Copied => (IconName::Check, "LINK COPIED", Some(GREEN)),
            ShareState::Failed => (IconName::CircleAlert, "SHARE FAILED", Some(RED)),
        };

        div()
            .id("share-scan")
            .w_full()
            .py_2()
            .flex()
            .items_center()
            .justify_center()
            .gap_2()
            .rounded_sm()
            .border_1()
            .bg(theme::color(BG_2))
            .text_xs()
            .font_weight(FontWeight::MEDIUM)
            .map(|el| match accent {
                Some(accent) => el
                    .border_color(theme::tint(accent, 0x80))
                    .text_color(theme::color(accent)),
                None => el
                    .border_color(theme::color(BORDER))
                    .text_color(theme::color(TEXT_2)),
            })
            .when(disabled, |el| el.opacity(0.5))
            .when(!disabled, |el| {
                el.cursor_pointer()
                    .hover(|s| {
                        s.border_color(theme::color(CYAN_DIM))
                            .text_color(theme::color(CYAN))
                    })
                    .on_click(cx.listener(|this, _, _, cx| this.share(cx)))
            })
            .child(Icon::new(icon).size(px(14.)))
            .child(label)
    }
}
