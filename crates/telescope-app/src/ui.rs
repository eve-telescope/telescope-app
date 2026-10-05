//! Small presentational helpers shared across views.

use gpui_kit::{
    Div, FontWeight, IntoElement, ParentElement as _, SharedString, Styled as _, div, px,
};

pub use gpui_kit::assets::IconName;
pub use gpui_kit::component::Icon;

use crate::theme::{self, TEXT_3};

/// The small letter-spaced uppercase heading used above panel sections.
pub fn section_title(text: impl Into<SharedString>) -> Div {
    div()
        .text_size(px(10.))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(theme::color(TEXT_3))
        .child(text.into())
}

pub fn mono() -> SharedString {
    if cfg!(target_os = "macos") {
        "Menlo".into()
    } else if cfg!(target_os = "windows") {
        "Consolas".into()
    } else {
        "DejaVu Sans Mono".into()
    }
}

pub fn dot(color: gpui_kit::Hsla, size: f32) -> impl IntoElement {
    div().size(px(size)).rounded_full().bg(color).flex_none()
}
