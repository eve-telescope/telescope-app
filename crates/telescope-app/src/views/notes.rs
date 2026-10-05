//! The sticky-note button on a pilot row, listing every annotation note that
//! applies to the pilot, their corporation or their alliance.

use gpui_kit::component::Sizable as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::popover::Popover;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{AnyElement, FontWeight, IntoElement, ParentElement as _, Styled as _, div, px};
use telescope_core::models::PilotIntel;
use telescope_core::view::annotations::{
    Annotation, DEFAULT_ANNOTATION_COLOR, ResolvedAnnotation, Target,
};

use crate::theme::{self, BG_3, ORANGE, TEXT_1, TEXT_2, TEXT_3};
use crate::ui::{Icon, IconName};
use crate::views::intel_card::{divider, scope_pill, target_avatar};

fn note_card(annotation: &Annotation) -> impl IntoElement {
    let color = annotation
        .color
        .clone()
        .unwrap_or_else(|| DEFAULT_ANNOTATION_COLOR.into());
    div()
        .flex()
        .flex_col()
        .gap_1p5()
        .py_2()
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(target_avatar(&Target::of(annotation), 20.))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme::color(TEXT_1))
                        .child(annotation.target_name.clone()),
                )
                .child(scope_pill(annotation.target_type)),
        )
        .when(!annotation.tags.is_empty(), |el| {
            el.child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_1()
                    .children(annotation.tags.iter().map(|tag| {
                        div()
                            .px_1p5()
                            .rounded_full()
                            .text_size(px(10.))
                            .font_weight(FontWeight::BOLD)
                            .bg(theme::hex_tint(&color, 0x26, BG_3))
                            .text_color(theme::hex_or(&color, TEXT_2))
                            .child(tag.clone())
                    })),
            )
        })
        .child(
            div()
                .text_size(px(11.))
                .text_color(theme::color(TEXT_2))
                .child(annotation.note.clone().unwrap_or_default()),
        )
        .child(
            div()
                .text_size(px(9.))
                .text_color(theme::color(TEXT_3))
                .child(annotation.network_name.clone()),
        )
}

pub fn annotation_notes_button(
    pilot: &PilotIntel,
    resolved: &[ResolvedAnnotation<'_>],
) -> Option<AnyElement> {
    let notes: Vec<Annotation> = resolved
        .iter()
        .filter(|r| {
            r.annotation
                .note
                .as_deref()
                .is_some_and(|n| !n.trim().is_empty())
        })
        .map(|r| r.annotation.clone())
        .collect();
    if notes.is_empty() {
        return None;
    }
    let id = pilot.character.id as u64;
    Some(
        Popover::new(("notes", id))
            .trigger(
                Button::new(("notes-trigger", id))
                    .icon(Icon::new(IconName::StickyNote).text_color(theme::color(ORANGE)))
                    .tooltip(format!("{} note(s)", notes.len()))
                    .ghost()
                    .xsmall(),
            )
            .child(
                div()
                    .w(px(300.))
                    .flex()
                    .flex_col()
                    .children(notes.iter().enumerate().flat_map(|(i, note)| {
                        let line = (i > 0).then(|| divider().into_any_element());
                        line.into_iter()
                            .chain(std::iter::once(note_card(note).into_any_element()))
                    })),
            )
            .into_any_element(),
    )
}
