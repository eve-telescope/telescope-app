//! The right-click intel panel for a pilot: one card per character,
//! corporation and alliance, with clickable tag chips and the note.

use std::rc::Rc;

use gpui_kit::component::Sizable as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AnyElement, App, Div, ElementId, FontWeight, Hsla, InteractiveElement as _, IntoElement,
    ParentElement as _, SharedString, Stateful, StatefulInteractiveElement as _, Styled as _,
    Window, div, img, px,
};
use telescope_core::models::PilotIntel;
use telescope_core::view::annotations::{
    Annotation, DEFAULT_ANNOTATION_COLOR, EntityType, PRESET_ANNOTATION_TAGS, Target,
    find_annotation,
};
use telescope_core::view::network::portrait_url;

use crate::state::Stores;
use crate::theme::{self, BG_1, BG_2, BG_3, BORDER, TEXT_1, TEXT_2, TEXT_3};
use crate::ui::{Icon, IconName, mono};
use crate::views::annotation_form;
use crate::views::local_panel::threat_badge;

/// Border for surfaces that float above the table.
const RAISED_BORDER: u32 = 0x3a3a46;

pub type Close = Rc<dyn Fn(&mut Window, &mut App)>;

/// Label and accent for each annotation scope.
pub fn scope_style(scope: EntityType) -> (&'static str, u32) {
    match scope {
        EntityType::Character => ("CHAR", 0x38bdf8),
        EntityType::Corporation => ("CORP", 0xa78bfa),
        EntityType::Alliance => ("ALLY", 0xfbbf24),
    }
}

pub fn scope_pill(scope: EntityType) -> Div {
    let (label, color) = scope_style(scope);
    div()
        .flex_none()
        .px_1p5()
        .rounded_sm()
        .bg(theme::tint(color, 0x26))
        .text_color(theme::color(color))
        .text_size(px(9.))
        .font_weight(FontWeight::BOLD)
        .child(label)
}

pub fn target_avatar(target: &Target, size: f32) -> AnyElement {
    match portrait_url(target.entity_type.as_str(), target.id, 64) {
        Some(url) => img(url)
            .size(px(size))
            .flex_none()
            .rounded_md()
            .bg(theme::color(BG_3))
            .into_any_element(),
        None => div()
            .size(px(size))
            .flex_none()
            .rounded_md()
            .bg(theme::color(BG_3))
            .into_any_element(),
    }
}

/// A tag chip that toggles: filled with the tag color when applied,
/// outlined otherwise.
pub fn toggle_chip(id: ElementId, tag: &str, color: &str, active: bool) -> Stateful<Div> {
    let accent: Hsla = theme::hex_or(color, TEXT_2);
    div()
        .id(id)
        .flex_none()
        .px_2()
        .py_0p5()
        .rounded_full()
        .border_1()
        .text_size(px(10.))
        .font_weight(FontWeight::BOLD)
        .cursor_pointer()
        .map(|el| {
            if active {
                el.bg(theme::hex_tint(color, 0x33, TEXT_3))
                    .border_color(accent)
                    .text_color(accent)
            } else {
                el.border_color(theme::color(BORDER))
                    .text_color(theme::color(TEXT_3))
                    .hover(move |s| s.border_color(accent.opacity(0.6)).text_color(accent))
            }
        })
        .child(tag.to_string())
}

/// Preset tags, then the network's custom tags, then any other tag already
/// on this annotation.
pub fn tag_options(applied: &[String], customs: &[(String, String)]) -> Vec<(String, String)> {
    let mut options: Vec<(String, String)> = PRESET_ANNOTATION_TAGS
        .iter()
        .map(|p| (p.tag.to_string(), p.color.to_string()))
        .chain(customs.iter().cloned())
        .collect();
    for tag in applied {
        if !options.iter().any(|(t, _)| t == tag) {
            options.push((tag.clone(), DEFAULT_ANNOTATION_COLOR.to_string()));
        }
    }
    options
}

fn target_card(
    network_id: i64,
    target: Target,
    existing: Option<Annotation>,
    customs: &[(String, String)],
    close: Close,
) -> impl IntoElement {
    let applied: Vec<String> = existing
        .as_ref()
        .map(|a| a.tags.clone())
        .unwrap_or_default();
    let note = existing
        .as_ref()
        .and_then(|a| a.note.clone())
        .filter(|n| !n.trim().is_empty());
    let scope = scope_style(target.entity_type).0;

    div()
        .flex()
        .flex_col()
        .gap_2()
        .p_2p5()
        .rounded_md()
        .bg(theme::color(BG_1))
        .border_1()
        .border_color(theme::color(BORDER))
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(target_avatar(&target, 24.))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme::color(TEXT_1))
                        .child(target.name.clone()),
                )
                .child(scope_pill(target.entity_type))
                .child({
                    let target = target.clone();
                    let existing = existing.clone();
                    Button::new(SharedString::from(format!("note-{scope}")))
                        .ghost()
                        .xsmall()
                        .icon(IconName::Pencil)
                        .tooltip("Edit tags and note")
                        .on_click(move |_, window, cx| {
                            close(window, cx);
                            annotation_form::open(
                                network_id,
                                Some(target.clone()),
                                existing.as_ref(),
                                window,
                                cx,
                            );
                        })
                }),
        )
        .child(
            div().flex().flex_wrap().gap_1().children(
                tag_options(&applied, customs)
                    .into_iter()
                    .map(|(tag, color)| {
                        let active = applied.contains(&tag);
                        let target = target.clone();
                        toggle_chip(
                            SharedString::from(format!("chip-{scope}-{tag}")).into(),
                            &tag,
                            &color,
                            active,
                        )
                        .on_click(move |_, _, cx| {
                            let target = target.clone();
                            Stores::get(cx).intel.update(cx, |intel, cx| {
                                intel.toggle_tag(network_id, target, &tag, cx)
                            });
                        })
                    }),
            ),
        )
        .when_some(note, |el, note| {
            el.child(
                div()
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .text_xs()
                    .text_color(theme::color(TEXT_2))
                    .child(
                        Icon::new(IconName::StickyNote)
                            .size_3()
                            .text_color(theme::color(TEXT_3)),
                    )
                    .child(div().flex_1().min_w_0().truncate().child(note)),
            )
        })
}

pub fn intel_panel(pilot: &PilotIntel, close: Close, cx: &App) -> impl IntoElement {
    let intel = Stores::get(cx).intel.read(cx);
    let network = intel
        .active_network()
        .filter(|_| intel.is_authenticated())
        .map(|n| (n.id, n.name.clone()));
    let c = &pilot.character;
    let affiliation = [
        c.corporation_ticker.as_ref().map(|t| format!("[{t}]")),
        c.alliance_ticker.as_ref().map(|t| format!("<{t}>")),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" ");

    let header = div()
        .flex()
        .items_center()
        .gap_2p5()
        .child(target_avatar(
            &Target {
                entity_type: EntityType::Character,
                id: c.id,
                name: c.name.clone(),
            },
            40.,
        ))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .child(
                    div()
                        .truncate()
                        .text_base()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme::color(TEXT_1))
                        .child(c.name.clone()),
                )
                .child(
                    div()
                        .truncate()
                        .font_family(mono())
                        .text_xs()
                        .text_color(theme::color(TEXT_3))
                        .child(affiliation),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .items_end()
                .gap_0p5()
                .child(threat_badge(&pilot.threat_level))
                .when_some(pilot.danger, |el, danger| {
                    el.child(
                        div()
                            .font_family(mono())
                            .text_xs()
                            .text_color(theme::threat_color(&pilot.threat_level))
                            .child(format!("{danger} danger")),
                    )
                }),
        );

    let body = match network {
        None => div()
            .p_3()
            .rounded_md()
            .bg(theme::color(BG_1))
            .border_1()
            .border_color(theme::color(BORDER))
            .text_xs()
            .text_color(theme::color(TEXT_2))
            .child("Connect to an intel network in the Network tab to tag pilots.")
            .into_any_element(),
        Some((network_id, name)) => {
            let customs = intel.network_custom_tags(network_id);
            div()
                .flex()
                .flex_col()
                .gap_2()
                .children(Target::for_pilot(pilot).into_iter().map(|target| {
                    let existing =
                        find_annotation(intel.annotation_index(), network_id, &target).cloned();
                    target_card(network_id, target, existing, &customs, close.clone())
                }))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .text_size(px(10.))
                        .text_color(theme::color(TEXT_3))
                        .child(Icon::new(IconName::Network).size_3())
                        .child(format!("Tagging on {name}")),
                )
                .into_any_element()
        }
    };

    div()
        .id("intel-panel")
        .w(px(380.))
        .flex()
        .flex_col()
        .gap_3()
        .p_3()
        .rounded_lg()
        .bg(theme::color(BG_2))
        .border_1()
        .border_color(theme::color(RAISED_BORDER))
        .shadow_lg()
        .occlude()
        .child(header)
        .child(body)
}
