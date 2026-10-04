//! Building blocks for settings pages: a page header, grouped lists of rows
//! on a raised surface, and rows with a label, description and control.

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AnyElement, Div, FontWeight, IntoElement, ParentElement as _, SharedString, Styled as _, div,
    px,
};

use crate::theme::{self, TEXT_1, TEXT_3};

/// The padded, width-limited column every page is laid out in.
pub fn page_body() -> Div {
    div()
        .w_full()
        .max_w(px(720.))
        .flex()
        .flex_col()
        .gap_6()
        .px_8()
        .py_7()
}

pub fn page(title: impl Into<SharedString>, subtitle: impl Into<SharedString>) -> Div {
    page_body().child(
        div()
            .flex()
            .flex_col()
            .gap_1()
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme::color(TEXT_1))
                    .child(title.into()),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(theme::color(TEXT_3))
                    .child(subtitle.into()),
            ),
    )
}

/// A titled list of rows separated by hairlines.
pub fn group(title: Option<&'static str>, rows: Vec<AnyElement>) -> Div {
    let mut list = div()
        .flex()
        .flex_col()
        .overflow_hidden()
        .rounded(px(10.))
        .bg(theme::color(theme::SURFACE))
        .border_1()
        .border_color(theme::hairline(0x14));
    for (i, row) in rows.into_iter().enumerate() {
        if i > 0 {
            list = list.child(div().h(px(1.)).mx_4().bg(theme::hairline(0x0f)));
        }
        list = list.child(row);
    }
    div()
        .flex()
        .flex_col()
        .gap_2()
        .when_some(title, |el, title| {
            el.child(
                div()
                    .px_1()
                    .text_size(px(11.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme::color(TEXT_3))
                    .child(title),
            )
        })
        .child(list)
}

/// A row with `label` and an optional description on the left; add the
/// control with `.child(..)`.
pub fn row(label: impl Into<SharedString>, description: Option<SharedString>) -> Div {
    div()
        .flex()
        .items_center()
        .gap_4()
        .px_4()
        .py_3()
        .min_h(px(52.))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap_0p5()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme::color(TEXT_1))
                        .child(label.into()),
                )
                .when_some(description, |el, description| {
                    el.child(
                        div()
                            .text_xs()
                            .text_color(theme::color(TEXT_3))
                            .child(description),
                    )
                }),
        )
}

/// A centered message for empty lists inside a group.
pub fn empty_row(text: impl Into<SharedString>) -> AnyElement {
    div()
        .px_4()
        .py_6()
        .flex()
        .justify_center()
        .text_xs()
        .text_color(theme::color(TEXT_3))
        .child(text.into())
        .into_any_element()
}
