//! One-line tag display for table rows: a few chips, then a "+N" chip whose
//! tooltip lists every tag.

use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AnyElement, Div, ElementId, FontWeight, InteractiveElement as _, IntoElement,
    ParentElement as _, Pixels, StatefulInteractiveElement as _, Styled as _, div, px,
};
use telescope_core::view::pilot_tags::{DEFAULT_TAG_COLOR, DEFAULT_TAG_TEXT_COLOR, PilotTag};

use crate::theme::{self, BG_3, TEXT_2, TEXT_3};

#[derive(Clone, Copy)]
pub struct TagStyle {
    pub text_size: Pixels,
    pub max_chip_width: Pixels,
    pub visible: usize,
}

pub const TABLE: TagStyle = TagStyle {
    text_size: px(10.),
    max_chip_width: px(84.),
    visible: 2,
};

pub const OVERLAY: TagStyle = TagStyle {
    text_size: px(8.),
    max_chip_width: px(48.),
    visible: 1,
};

/// Annotation tags come from the user's network, so they lead; zKill flags
/// follow.
fn ordered(tags: &[PilotTag]) -> Vec<PilotTag> {
    let (flags, notes): (Vec<_>, Vec<_>) = tags
        .iter()
        .cloned()
        .partition(|t| t.key.starts_with("flag:"));
    notes.into_iter().chain(flags).collect()
}

pub fn chip(tag: &PilotTag, style: TagStyle) -> Div {
    let color = tag.color.as_deref().unwrap_or(DEFAULT_TAG_COLOR);
    let text = tag.color.as_deref().unwrap_or(DEFAULT_TAG_TEXT_COLOR);
    div()
        .flex_none()
        .max_w(style.max_chip_width)
        .px_1p5()
        .rounded_sm()
        .truncate()
        .text_size(style.text_size)
        .font_weight(FontWeight::SEMIBOLD)
        .bg(theme::hex_tint(color, 0x22, TEXT_3))
        .text_color(theme::hex_or(text, TEXT_2))
        .child(tag.text.clone())
}

pub fn tag_strip(id: ElementId, tags: &[PilotTag], style: TagStyle) -> AnyElement {
    let tags = ordered(tags);
    let hidden = tags.len().saturating_sub(style.visible);
    let all = tags.clone();

    div()
        .flex()
        .items_center()
        .gap_1()
        .min_w_0()
        .overflow_hidden()
        .children(tags.iter().take(style.visible).map(|t| chip(t, style)))
        .when(hidden > 0, move |el| {
            el.child(
                div()
                    .id(id)
                    .flex_none()
                    .px_1()
                    .rounded_sm()
                    .bg(theme::color(BG_3))
                    .text_size(style.text_size)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme::color(TEXT_2))
                    .child(format!("+{hidden}"))
                    .tooltip(move |window, cx| {
                        let all = all.clone();
                        Tooltip::element(move |_, _| {
                            div()
                                .max_w(px(260.))
                                .flex()
                                .flex_wrap()
                                .gap_1()
                                .children(all.iter().map(|t| chip(t, TABLE).max_w(px(240.))))
                        })
                        .build(window, cx)
                    }),
            )
        })
        .into_any_element()
}
