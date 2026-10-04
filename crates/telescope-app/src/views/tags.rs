//! One-line tag display for table rows. Roles are drawn as tinted hull
//! brackets and network annotations as text chips; whatever doesn't fit goes
//! into a "+N" chip whose tooltip lists every tag.

use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AnyElement, AnyView, App, Div, ElementId, FontWeight, InteractiveElement as _, IntoElement,
    ParentElement as _, Pixels, SharedString, StatefulInteractiveElement as _, Styled as _, Window,
    div, img, px,
};
use telescope_core::view::pilot_tags::{DEFAULT_TAG_COLOR, DEFAULT_TAG_TEXT_COLOR, PilotTag};
use telescope_core::view::roles::Role;

use crate::assets::bracket;
use crate::theme::{self, BG_3, TEXT_2, TEXT_3};

#[derive(Clone, Copy)]
pub struct TagStyle {
    pub text_size: Pixels,
    pub max_chip_width: Pixels,
    pub icon_size: Pixels,
    /// Annotation chips shown before the overflow chip.
    pub chips: usize,
    /// Role icons shown before the overflow chip.
    pub icons: usize,
}

pub const TABLE: TagStyle = TagStyle {
    text_size: px(10.),
    max_chip_width: px(84.),
    icon_size: px(24.),
    chips: 1,
    icons: 3,
};

pub const OVERLAY: TagStyle = TagStyle {
    text_size: px(8.),
    max_chip_width: px(40.),
    icon_size: px(18.),
    chips: 1,
    icons: 2,
};

fn color_of(tag: &PilotTag) -> &str {
    tag.color.as_deref().unwrap_or(DEFAULT_TAG_COLOR)
}

/// The hull bracket for a role label in `color`, for filter chips that only
/// know the tag text.
pub fn role_glyph(label: &str, color: &str, size: Pixels) -> Option<AnyElement> {
    let stem = Role::from_label(label)?.icon()?;
    Some(
        img(bracket(stem, color))
            .size(size)
            .flex_none()
            .into_any_element(),
    )
}

/// The tag's hull bracket in its role color, if it has one.
pub fn role_icon(tag: &PilotTag, size: Pixels) -> Option<AnyElement> {
    let stem = tag.icon?;
    Some(
        img(bracket(stem, color_of(tag)))
            .size(size)
            .flex_none()
            .into_any_element(),
    )
}

pub fn chip(tag: &PilotTag, style: TagStyle) -> Div {
    let color = color_of(tag);
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

/// Icon and label together, for tooltips.
pub fn labeled(tag: &PilotTag, style: TagStyle) -> Div {
    div()
        .flex()
        .items_center()
        .gap_1()
        .children(role_icon(tag, px(14.)))
        .child(chip(tag, style))
}

fn text_tooltip(text: SharedString) -> impl Fn(&mut Window, &mut App) -> AnyView {
    move |window, cx| Tooltip::new(text.clone()).build(window, cx)
}

/// `prefix` and `owner` keep element ids unique per row.
pub fn tag_strip(
    prefix: &'static str,
    owner: u64,
    tags: &[PilotTag],
    style: TagStyle,
) -> AnyElement {
    let (roles, notes): (Vec<_>, Vec<_>) = tags.iter().cloned().partition(PilotTag::is_role);
    let shown_notes = notes.len().min(style.chips);
    let shown_roles = roles.len().min(style.icons);
    let hidden = tags.len() - shown_notes - shown_roles;
    let all: Vec<PilotTag> = notes.iter().chain(&roles).cloned().collect();

    div()
        .flex()
        .items_center()
        .gap_1()
        .min_w_0()
        .overflow_hidden()
        .children(notes.iter().take(shown_notes).map(|t| chip(t, style)))
        .children(roles.iter().take(shown_roles).map(|t| {
            let id = ElementId::NamedInteger(format!("{prefix}-{}", t.key).into(), owner);
            let content = match role_icon(t, style.icon_size) {
                Some(icon) => icon,
                None => chip(t, style).into_any_element(),
            };
            div()
                .id(id)
                .flex_none()
                .tooltip(text_tooltip(t.text.clone().into()))
                .child(content)
        }))
        .when(hidden > 0, move |el| {
            el.child(
                div()
                    .id(ElementId::NamedInteger(
                        format!("{prefix}-more").into(),
                        owner,
                    ))
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
                                .max_w(px(280.))
                                .flex()
                                .flex_wrap()
                                .gap_1p5()
                                .children(all.iter().map(|t| labeled(t, TABLE)))
                        })
                        .build(window, cx)
                    }),
            )
        })
        .into_any_element()
}
