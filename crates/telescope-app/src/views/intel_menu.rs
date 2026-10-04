//! Right-click tagging for pilot rows, and the notes popover next to a row's
//! tags.

use gpui_kit::component::Sizable as _;
use gpui_kit::component::button::ButtonVariants as _;
use gpui_kit::component::menu::{ContextMenuExt as _, PopupMenu, PopupMenuItem};
use gpui_kit::component::popover::Popover;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AnyElement, App, Div, FontWeight, InteractiveElement as _, IntoElement, ParentElement as _,
    Stateful, Styled as _, Window, div, px,
};
use telescope_core::models::PilotIntel;
use telescope_core::view::annotations::{
    Annotation, DEFAULT_ANNOTATION_COLOR, EntityType, PRESET_ANNOTATION_TAGS, ResolvedAnnotation,
    format_scope, normalize_annotation_tag,
};

use crate::state::Stores;
use crate::state::intel::IntelStore;
use crate::theme::{self, BG_3, ORANGE, TEXT_2, TEXT_3};
use crate::ui::{Icon, IconName};
use crate::views::annotation_form::{self, Target};

#[derive(Clone)]
struct MenuTarget {
    target: Target,
    icon: IconName,
}

fn targets(pilot: &PilotIntel) -> Vec<MenuTarget> {
    let c = &pilot.character;
    let mut targets = vec![MenuTarget {
        target: Target {
            entity_type: EntityType::Character,
            id: c.id,
            name: c.name.clone(),
        },
        icon: IconName::User,
    }];
    if let (Some(id), Some(name)) = (c.corporation_id.filter(|id| *id != 0), &c.corporation_name) {
        targets.push(MenuTarget {
            target: Target {
                entity_type: EntityType::Corporation,
                id,
                name: name.clone(),
            },
            icon: IconName::Building2,
        });
    }
    if let (Some(id), Some(name)) = (c.alliance_id.filter(|id| *id != 0), &c.alliance_name) {
        targets.push(MenuTarget {
            target: Target {
                entity_type: EntityType::Alliance,
                id,
                name: name.clone(),
            },
            icon: IconName::Flag,
        });
    }
    targets
}

fn find_annotation<'a>(
    intel: &'a IntelStore,
    network_id: i64,
    target: &Target,
) -> Option<&'a Annotation> {
    intel
        .annotation_index()
        .get(&(target.entity_type, target.id))?
        .iter()
        .find(|a| a.network_id == network_id)
}

/// Tags used on the active network that aren't presets, with their color.
fn custom_tags(intel: &IntelStore, network_id: i64) -> Vec<(String, String)> {
    let mut tags: Vec<(String, String)> = Vec::new();
    for annotation in intel.annotations().filter(|a| a.network_id == network_id) {
        for tag in &annotation.tags {
            let preset = PRESET_ANNOTATION_TAGS.iter().any(|p| p.tag == tag);
            if !preset && !tags.iter().any(|(t, _)| t == tag) {
                let color = annotation
                    .color
                    .clone()
                    .unwrap_or_else(|| DEFAULT_ANNOTATION_COLOR.into());
                tags.push((tag.clone(), color));
            }
        }
    }
    tags.sort_by(|a, b| a.0.cmp(&b.0));
    tags
}

fn quick_toggle(network_id: i64, target: &Target, tag: &str, cx: &mut App) {
    let stores = Stores::get(cx);
    let existing = find_annotation(stores.intel.read(cx), network_id, target).cloned();
    let tag = normalize_annotation_tag(tag);
    let target = target.clone();
    stores.intel.update(cx, |intel, cx| match existing {
        Some(existing) => {
            let mut tags = existing.tags.clone();
            match tags.iter().position(|t| *t == tag) {
                Some(i) => {
                    tags.remove(i);
                }
                None => tags.push(tag),
            }
            if tags.is_empty() {
                intel.remove_entry(network_id, existing.id, cx);
            } else {
                intel.save_annotation(
                    network_id,
                    Some(existing.id),
                    target.entity_type,
                    target.id,
                    target.name,
                    tags,
                    existing.note,
                    cx,
                );
            }
        }
        None => intel.save_annotation(
            network_id,
            None,
            target.entity_type,
            target.id,
            target.name,
            vec![tag],
            None,
            cx,
        ),
    });
}

fn target_submenu(
    menu: PopupMenu,
    network_id: i64,
    entry: MenuTarget,
    customs: Vec<(String, String)>,
    applied: Vec<String>,
) -> PopupMenu {
    let target = entry.target.clone();
    let mut menu = menu
        .label(format!(
            "{} · {}",
            target.name,
            format_scope(target.entity_type)
        ))
        .separator();
    for preset in PRESET_ANNOTATION_TAGS.iter() {
        let target = target.clone();
        let tag = preset.tag;
        menu = menu.item(
            PopupMenuItem::new(tag)
                .checked(applied.iter().any(|t| t == tag))
                .on_click(move |_, _, cx| quick_toggle(network_id, &target, tag, cx)),
        );
    }
    if !customs.is_empty() {
        menu = menu.separator();
        for (tag, _) in customs {
            let target = target.clone();
            let checked = applied.contains(&tag);
            menu = menu.item(
                PopupMenuItem::new(tag.clone())
                    .checked(checked)
                    .on_click(move |_, _, cx| quick_toggle(network_id, &target, &tag, cx)),
            );
        }
    }
    menu.separator().item(
        PopupMenuItem::new("Annotation...").on_click(move |_, window, cx| {
            let intel = Stores::get(cx).intel.read(cx);
            let existing = find_annotation(intel, network_id, &target).cloned();
            annotation_form::open(
                network_id,
                Some(target.clone()),
                existing.as_ref(),
                window,
                cx,
            );
        }),
    )
}

pub fn pilot_context_menu(
    row: Stateful<Div>,
    pilot: &PilotIntel,
    _window: &mut Window,
    _cx: &mut App,
) -> AnyElement {
    let targets = targets(pilot);
    row.context_menu(move |menu, window, cx| {
        let intel = Stores::get(cx).intel.read(cx);
        let Some(network) = intel
            .active_network()
            .filter(|_| intel.is_authenticated())
            .map(|n| n.id)
        else {
            return menu.item(PopupMenuItem::new("No active network").disabled(true));
        };
        let customs = custom_tags(intel, network);
        let applied: Vec<Vec<String>> = targets
            .iter()
            .map(|t| {
                find_annotation(intel, network, &t.target)
                    .map(|a| a.tags.clone())
                    .unwrap_or_default()
            })
            .collect();

        let mut menu = menu.min_w(px(200.));
        for (entry, applied) in targets.iter().cloned().zip(applied) {
            let label = match entry.target.entity_type {
                EntityType::Character => "Character",
                EntityType::Corporation => "Corporation",
                EntityType::Alliance => "Alliance",
            };
            let customs = customs.clone();
            let icon = entry.icon;
            menu = menu.submenu_with_icon(
                Some(Icon::new(icon)),
                label,
                window,
                cx,
                move |sub, _, _| {
                    target_submenu(
                        sub,
                        network,
                        entry.clone(),
                        customs.clone(),
                        applied.clone(),
                    )
                },
            );
        }
        menu
    })
    .into_any_element()
}

/// The sticky-note button listing every annotation with a note for `pilot`.
pub fn annotation_notes_button(
    pilot: &PilotIntel,
    resolved: &[ResolvedAnnotation<'_>],
    _cx: &App,
) -> Option<AnyElement> {
    let notes: Vec<(EntityType, Annotation)> = resolved
        .iter()
        .filter(|r| {
            r.annotation
                .note
                .as_deref()
                .is_some_and(|n| !n.trim().is_empty())
        })
        .map(|r| (r.scope, r.annotation.clone()))
        .collect();
    if notes.is_empty() {
        return None;
    }
    let count = notes.len();
    Some(
        Popover::new(("notes", pilot.character.id as u64))
            .trigger(
                gpui_kit::component::button::Button::new((
                    "notes-trigger",
                    pilot.character.id as u64,
                ))
                .icon(Icon::new(IconName::StickyNote).text_color(theme::color(ORANGE)))
                .tooltip(format!("{count} note(s)"))
                .ghost()
                .xsmall(),
            )
            .child(
                div()
                    .w(px(280.))
                    .flex()
                    .flex_col()
                    .gap_2()
                    .children(notes.into_iter().map(|(scope, annotation)| {
                        let color = annotation
                            .color
                            .clone()
                            .unwrap_or_else(|| DEFAULT_ANNOTATION_COLOR.into());
                        let badge = |text: String| {
                            div()
                                .px_1p5()
                                .rounded_sm()
                                .text_size(px(10.))
                                .font_weight(FontWeight::SEMIBOLD)
                                .bg(theme::hex_tint(&color, 0x22, BG_3))
                                .text_color(theme::hex_or(&color, TEXT_2))
                                .child(text)
                        };
                        div()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1p5()
                                    .mb_1()
                                    .child(badge(format_scope(scope).to_string()))
                                    .child(
                                        div()
                                            .text_size(px(9.))
                                            .text_color(theme::color(TEXT_3))
                                            .child(annotation.network_name.clone()),
                                    )
                                    .child(
                                        div()
                                            .truncate()
                                            .text_size(px(9.))
                                            .text_color(theme::color(TEXT_3))
                                            .child(annotation.target_name.clone()),
                                    ),
                            )
                            .when(!annotation.tags.is_empty(), |el| {
                                el.child(
                                    div()
                                        .flex()
                                        .flex_wrap()
                                        .gap_1()
                                        .mb_1()
                                        .children(annotation.tags.iter().cloned().map(badge)),
                                )
                            })
                            .child(
                                div()
                                    .text_size(px(11.))
                                    .text_color(theme::color(TEXT_2))
                                    .child(annotation.note.clone().unwrap_or_default()),
                            )
                    })),
            )
            .into_any_element(),
    )
}
