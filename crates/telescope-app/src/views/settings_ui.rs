//! Building blocks for settings pages: a page header, flat sections of rows
//! separated by hairlines, and tables with a header row and aligned columns.

use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AnyElement, Div, ElementId, FontWeight, InteractiveElement as _, IntoElement,
    ParentElement as _, SharedString, Stateful, Styled as _, div, px,
};

use crate::theme::{self, TEXT_1, TEXT_3};
use crate::views::grid::{Col, cell};

/// The padded, width-limited column every page is laid out in.
pub fn page_body() -> Div {
    div()
        .w_full()
        .max_w(px(880.))
        .flex()
        .flex_col()
        .gap_8()
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

fn hairline() -> Div {
    div().h(px(1.)).flex_none().bg(theme::hairline(0x14))
}

fn section_title(title: &'static str) -> Div {
    div()
        .pb_2()
        .text_size(px(11.))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(theme::color(TEXT_3))
        .child(title)
}

/// A titled stack of rows, each followed by a hairline.
pub fn group(title: Option<&'static str>, rows: Vec<AnyElement>) -> Div {
    let mut section = div()
        .flex()
        .flex_col()
        .when_some(title, |el, title| el.child(section_title(title)))
        .child(hairline());
    for row in rows {
        section = section.child(row).child(hairline());
    }
    section
}

/// A row with `label` and an optional description on the left; add the
/// control with `.child(..)`.
pub fn row(label: impl Into<SharedString>, description: Option<SharedString>) -> Div {
    div()
        .flex()
        .items_center()
        .gap_4()
        .py_3()
        .min_h(px(56.))
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

/// A message spanning the table for empty lists.
pub fn empty_row(text: impl Into<SharedString>) -> AnyElement {
    div()
        .py_6()
        .flex()
        .justify_center()
        .text_xs()
        .text_color(theme::color(TEXT_3))
        .child(text.into())
        .into_any_element()
}

#[derive(Clone, Copy)]
pub struct Column {
    pub label: &'static str,
    pub width: Col,
    pub right: bool,
}

impl Column {
    pub const fn new(label: &'static str, width: Col) -> Self {
        Self {
            label,
            width,
            right: false,
        }
    }

    pub const fn right(mut self) -> Self {
        self.right = true;
        self
    }
}

/// A table cell sized for `column`.
pub fn table_cell(column: Column) -> Div {
    cell(column.width)
        .px_2()
        .when(column.right, |el| el.justify_end())
}

/// A clickable or hoverable table row; fill it with one `table_cell` per
/// column.
pub fn table_row(id: impl Into<ElementId>) -> Stateful<Div> {
    div()
        .id(id)
        .flex()
        .items_center()
        .min_h(px(44.))
        .py_1p5()
        .hover(|s| s.bg(theme::hairline(0x06)))
}

pub fn table(columns: &[Column], rows: Vec<AnyElement>) -> Div {
    let header = div()
        .flex()
        .items_center()
        .pb_2()
        .children(columns.iter().map(|column| {
            table_cell(*column)
                .text_size(px(11.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme::color(TEXT_3))
                .child(column.label)
        }));
    let mut table = div().flex().flex_col().child(header).child(hairline());
    for row in rows {
        table = table.child(row).child(hairline());
    }
    table
}
