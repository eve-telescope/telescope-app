//! The right-click intel panel for a pilot: a flat surface with a header and
//! one section per character, corporation and alliance, each with clickable
//! tag chips and the note.

use std::rc::Rc;

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
use crate::theme::{self, BG_3, TEXT_1, TEXT_2, TEXT_3};
use crate::ui::{Icon, IconName, mono};
use crate::views::annotation_form;

pub type Close = Rc<dyn Fn(&mut Window, &mut App)>;

/// Label and accent for each annotation scope.
pub fn scope_style(scope: EntityType) -> (&'static str, u32) {
    match scope {
        EntityType::Character => ("CHARACTER", 0x38bdf8),
        EntityType::Corporation => ("CORPORATION", 0xa78bfa),
        EntityType::Alliance => ("ALLIANCE", 0xfbbf24),
    }
}

/// The scope as small colored caps.
pub fn scope_pill(scope: EntityType) -> Div {
    let (label, color) = scope_style(scope);
    div()
        .flex_none()
        .text_size(px(9.))
        .font_weight(FontWeight::BOLD)
        .text_color(theme::color(color).opacity(0.85))
        .child(label)
}

pub fn target_avatar(target: &Target, size: f32) -> AnyElement {
    match portrait_url(target.entity_type.as_str(), target.id, 64) {
        Some(url) => img(url)
            .size(px(size))
            .flex_none()
            .rounded(px(size / 4.))
            .bg(theme::color(BG_3))
            .into_any_element(),
        None => div()
            .size(px(size))
            .flex_none()
            .rounded(px(size / 4.))
            .bg(theme::color(BG_3))
            .into_any_element(),
    }
}

/// A tag that toggles: tinted in its color when applied, a faint neutral
/// fill otherwise.
pub fn toggle_chip(id: ElementId, tag: &str, color: &str, active: bool) -> Stateful<Div> {
    let accent: Hsla = theme::hex_or(color, TEXT_2);
    div()
        .id(id)
        .flex_none()
        .px_2()
        .py(px(3.))
        .rounded_full()
        .text_size(px(10.))
        .font_weight(FontWeight::SEMIBOLD)
        .cursor_pointer()
        .map(|el| {
            if active {
                el.bg(accent.opacity(0.18)).text_color(accent)
            } else {
                el.bg(theme::hairline(0x0a))
                    .text_color(theme::color(TEXT_3))
                    .hover(move |s| s.bg(accent.opacity(0.1)).text_color(accent))
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

pub fn divider() -> Div {
    div().h(px(1.)).flex_none().bg(theme::hairline(0x0f))
}

fn target_section(
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
    let group: SharedString = format!("section-{scope}").into();

    div()
        .group(group.clone())
        .flex()
        .flex_col()
        .gap_2()
        .px_3()
        .py_2p5()
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(target_avatar(&target, 18.))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .truncate()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme::color(TEXT_1))
                                .child(target.name.clone()),
                        )
                        .child(scope_pill(target.entity_type)),
                )
                .child({
                    let target = target.clone();
                    let existing = existing.clone();
                    div()
                        .id(SharedString::from(format!("edit-{scope}")))
                        .flex_none()
                        .p_1()
                        .rounded_sm()
                        .text_color(theme::color(TEXT_3))
                        .invisible()
                        .group_hover(group, |s| s.visible())
                        .cursor_pointer()
                        .hover(|s| s.bg(theme::hairline(0x14)).text_color(theme::color(TEXT_1)))
                        .child(Icon::new(IconName::Pencil).size_3())
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
                    .pl_2()
                    .border_l_2()
                    .border_color(theme::hairline(0x1f))
                    .text_xs()
                    .text_color(theme::color(TEXT_2))
                    .line_clamp(2)
                    .child(note),
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
    let threat = theme::threat_color(&pilot.threat_level);

    let header = div()
        .flex()
        .items_center()
        .gap_2p5()
        .px_3()
        .py_3()
        .child(target_avatar(
            &Target {
                entity_type: EntityType::Character,
                id: c.id,
                name: c.name.clone(),
            },
            36.,
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
                .child(
                    div()
                        .text_size(px(9.))
                        .font_weight(FontWeight::BOLD)
                        .text_color(threat)
                        .child(pilot.threat_level.to_uppercase()),
                )
                .when_some(pilot.danger, |el, danger| {
                    el.child(
                        div()
                            .font_family(mono())
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(threat)
                            .child(danger.to_string()),
                    )
                }),
        );

    let body = match network {
        None => div()
            .px_3()
            .py_3()
            .text_xs()
            .text_color(theme::color(TEXT_2))
            .child("Connect to an intel network in Settings to tag pilots.")
            .into_any_element(),
        Some((network_id, name)) => {
            let customs = intel.network_custom_tags(network_id);
            let mut body = div().flex().flex_col();
            for (i, target) in Target::for_pilot(pilot).into_iter().enumerate() {
                if i > 0 {
                    body = body.child(divider().mx_3());
                }
                let existing =
                    find_annotation(intel.annotation_index(), network_id, &target).cloned();
                body = body.child(target_section(
                    network_id,
                    target,
                    existing,
                    &customs,
                    close.clone(),
                ));
            }
            body.child(divider())
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .px_3()
                        .py_2()
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
        .w(px(340.))
        .flex()
        .flex_col()
        .overflow_hidden()
        .rounded(px(10.))
        .bg(theme::color(theme::SURFACE))
        .border_1()
        .border_color(theme::hairline(0x14))
        .shadow_lg()
        .occlude()
        .child(header)
        .child(divider())
        .child(body)
}
