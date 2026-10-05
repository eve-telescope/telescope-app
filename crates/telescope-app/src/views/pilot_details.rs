use chrono::{Datelike, Timelike, Utc};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    App, Div, ElementId, FontWeight, Hsla, InteractiveElement as _, IntoElement,
    ParentElement as _, SharedString, StatefulInteractiveElement as _, Styled as _, div, img, px,
    relative,
};
use telescope_core::models::{ActivityHeatmap, PilotIntel, ZkillStats};
use telescope_core::view::format::{format_isk, ship_icon_url};

use crate::theme::{self, BG_2, BG_3, BORDER, CYAN, GREEN, ORANGE, RED, TEXT_1, TEXT_2, TEXT_3};
use crate::ui::{Icon, IconName, mono};
use crate::views::stats::group_thousands;

/// Height of the expanded panel, fixed so the virtual list can size rows.
pub const DETAILS_HEIGHT: f32 = 236.;

const DAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

fn heading(text: &'static str) -> Div {
    div()
        .mb_2()
        .pb_1()
        .border_b_1()
        .border_color(theme::color(BORDER))
        .text_size(px(10.))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(theme::color(TEXT_3))
        .child(text)
}

fn empty_note(text: &'static str) -> Div {
    div()
        .text_xs()
        .italic()
        .text_color(theme::color(TEXT_3))
        .child(text)
}

fn stat(label: &'static str, value: String) -> Div {
    div()
        .flex()
        .justify_between()
        .gap_4()
        .text_xs()
        .child(div().text_color(theme::color(TEXT_2)).child(label))
        .child(
            div()
                .font_family(mono())
                .text_color(theme::color(TEXT_1))
                .child(value),
        )
}

pub fn zkill_character_url(character_id: i64) -> String {
    format!("https://zkillboard.com/character/{character_id}/")
}

fn link_button(
    id: ElementId,
    label: &'static str,
    color: u32,
    tooltip: &'static str,
    url: String,
) -> impl IntoElement {
    div()
        .id(id)
        .px_1()
        .rounded_sm()
        .bg(theme::color(BG_3))
        .text_size(px(9.))
        .font_family(mono())
        .text_color(theme::color(color))
        .cursor_pointer()
        .hover(|s| s.bg(theme::tint(color, 0x33)))
        .tooltip(move |window, cx| {
            gpui_kit::component::tooltip::Tooltip::new(tooltip).build(window, cx)
        })
        .on_click(move |_, _, cx| {
            cx.stop_propagation();
            cx.open_url(&url)
        })
        .child(label)
}

fn combat(stats: &ZkillStats, character_id: i64) -> impl IntoElement {
    div()
        .child(
            div()
                .flex()
                .flex_col()
                .gap_0p5()
                .child(stat("Kills", group_thousands(stats.ships_destroyed)))
                .child(stat("Losses", group_thousands(stats.ships_lost)))
                .child(stat("ISK Destroyed", format_isk(stats.isk_destroyed)))
                .child(stat("ISK Lost", format_isk(stats.isk_lost)))
                .child(stat("Solo Kills", group_thousands(stats.solo_kills)))
                .child(stat("Danger", format!("{:.0}%", stats.danger_ratio)))
                .child(stat("Gang", format!("{:.0}%", stats.gang_ratio))),
        )
        .child(
            div()
                .id(("zkill", character_id as u64))
                .mt_3()
                .flex()
                .items_center()
                .gap_1()
                .text_size(px(10.))
                .text_color(theme::color(ORANGE))
                .cursor_pointer()
                .hover(|s| s.opacity(0.8))
                .on_click(move |_, _, cx| {
                    cx.stop_propagation();
                    cx.open_url(&zkill_character_url(character_id))
                })
                .child("zKillboard")
                .child(Icon::new(IconName::ExternalLink).size(px(10.))),
        )
}

fn ships(stats: &ZkillStats, character_id: i64) -> impl IntoElement {
    if stats.top_ships.is_empty() {
        return empty_note("No ship data").into_any_element();
    }
    div()
        .flex()
        .flex_col()
        .gap_1()
        .children(stats.top_ships.iter().take(5).map(|ship| {
            let base = format!(
                "https://zkillboard.com/character/{character_id}/ship/{}/",
                ship.ship_type_id
            );
            let group: SharedString = format!("ship-{}", ship.ship_type_id).into();
            div()
                .id(("ship", ship.ship_type_id as u64))
                .group(group.clone())
                .relative()
                .flex()
                .items_center()
                .gap_2()
                .text_xs()
                .child(
                    img(ship_icon_url(ship.ship_type_id, 32))
                        .size(px(20.))
                        .rounded_sm()
                        .bg(theme::color(BG_3)),
                )
                .child(
                    div()
                        .max_w(px(96.))
                        .truncate()
                        .child(ship.ship_name.clone()),
                )
                .child(
                    div()
                        .font_family(mono())
                        .text_size(px(11.))
                        .text_color(theme::color(GREEN))
                        .child(ship.kills.to_string()),
                )
                .child(
                    div()
                        .absolute()
                        .right_0()
                        .flex()
                        .gap_0p5()
                        .invisible()
                        .group_hover(group, |s| s.visible())
                        .child(link_button(
                            ("ship-k", ship.ship_type_id as u64).into(),
                            "K",
                            GREEN,
                            "View kills",
                            format!("{base}kills/"),
                        ))
                        .child(link_button(
                            ("ship-l", ship.ship_type_id as u64).into(),
                            "L",
                            RED,
                            "View losses",
                            format!("{base}losses/"),
                        ))
                        .child(link_button(
                            ("ship-b", ship.ship_type_id as u64).into(),
                            "B",
                            CYAN,
                            "View all",
                            base,
                        )),
                )
        }))
        .into_any_element()
}

fn systems(stats: &ZkillStats, character_id: i64) -> impl IntoElement {
    if stats.top_systems.is_empty() {
        return empty_note("No system data").into_any_element();
    }
    div()
        .flex()
        .flex_col()
        .gap_1()
        .children(stats.top_systems.iter().take(5).map(|system| {
            let base = format!(
                "https://zkillboard.com/character/{character_id}/system/{}/",
                system.system_id
            );
            let group: SharedString = format!("system-{}", system.system_id).into();
            div()
                .id(("system", system.system_id as u64))
                .group(group.clone())
                .relative()
                .flex()
                .items_center()
                .gap_2()
                .text_xs()
                .child(
                    div()
                        .max_w(px(120.))
                        .truncate()
                        .child(system.system_name.clone()),
                )
                .child(
                    div()
                        .font_family(mono())
                        .text_size(px(11.))
                        .text_color(theme::color(GREEN))
                        .child(system.kills.to_string()),
                )
                .child(
                    div()
                        .absolute()
                        .right_0()
                        .flex()
                        .gap_0p5()
                        .invisible()
                        .group_hover(group, |s| s.visible())
                        .child(link_button(
                            ("system-k", system.system_id as u64).into(),
                            "K",
                            GREEN,
                            "View kills",
                            format!("{base}kills/"),
                        ))
                        .child(link_button(
                            ("system-l", system.system_id as u64).into(),
                            "L",
                            RED,
                            "View losses",
                            format!("{base}losses/"),
                        ))
                        .child(link_button(
                            ("system-b", system.system_id as u64).into(),
                            "B",
                            CYAN,
                            "View all",
                            base,
                        )),
                )
        }))
        .into_any_element()
}

fn heat_color(value: i64, max: i64) -> Option<Hsla> {
    if value == 0 {
        return None;
    }
    let intensity = (value as f32 / max.max(1) as f32).min(1.);
    let mut color = theme::color(GREEN);
    color.a = 0.2 + intensity * 0.8;
    Some(color)
}

pub fn activity_heatmap(activity: &ActivityHeatmap, character_id: i64) -> impl IntoElement {
    let now = Utc::now();
    let today = now.weekday().num_days_from_sunday() as usize;
    let hour_fraction = (now.hour() as f32 + now.minute() as f32 / 60.) / 24.;
    let day_fraction = (today as f32 + 0.5) / 7.;
    let crosshair = theme::tint(ORANGE, 0x4d);

    div()
        .flex()
        .flex_col()
        .gap_0p5()
        .child(
            div()
                .flex()
                .items_center()
                .mb_0p5()
                .child(div().w(px(28.)))
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .justify_between()
                        .px_0p5()
                        .text_size(px(8.))
                        .text_color(theme::color(TEXT_3))
                        .children(["00", "06", "12", "18", "23"]),
                ),
        )
        .child(
            div()
                .relative()
                .children(activity.data.iter().enumerate().map(|(day, hours)| {
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(
                            div()
                                .w(px(24.))
                                .flex_none()
                                .text_size(px(9.))
                                .map(|el| {
                                    if day == today {
                                        el.text_color(theme::color(CYAN))
                                            .font_weight(FontWeight::SEMIBOLD)
                                    } else {
                                        el.text_color(theme::color(TEXT_3))
                                    }
                                })
                                .child(DAYS.get(day).copied().unwrap_or("")),
                        )
                        .child(div().flex_1().flex().gap(px(1.)).children(
                            hours.iter().enumerate().map(|(hour, &value)| {
                                let tooltip: SharedString = format!(
                                    "{} {:02}:00 - {} kills",
                                    DAYS.get(day).copied().unwrap_or(""),
                                    hour,
                                    value
                                )
                                .into();
                                div()
                                    .id(ElementId::NamedInteger(
                                        format!("heat-{character_id}-{day}").into(),
                                        hour as u64,
                                    ))
                                    .flex_1()
                                    .h(px(10.))
                                    .rounded_sm()
                                    .map(|el| match heat_color(value, activity.max) {
                                        Some(color) => el.bg(color),
                                        None => el.bg(theme::color(BG_3)),
                                    })
                                    .tooltip(move |window, cx| {
                                        gpui_kit::component::tooltip::Tooltip::new(tooltip.clone())
                                            .build(window, cx)
                                    })
                            }),
                        ))
                }))
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .bottom_0()
                        .left(px(28.))
                        .right_0()
                        .child(
                            div()
                                .absolute()
                                .top_0()
                                .bottom_0()
                                .w(px(1.))
                                .left(relative(hour_fraction))
                                .bg(crosshair),
                        )
                        .child(
                            div()
                                .absolute()
                                .left_0()
                                .right_0()
                                .h(px(1.))
                                .top(relative(day_fraction))
                                .bg(crosshair),
                        ),
                ),
        )
        .child(
            div()
                .mt_1()
                .flex()
                .justify_end()
                .font_family(mono())
                .text_size(px(9.))
                .text_color(theme::color(CYAN))
                .child(format!("{:02}:{:02} EVE", now.hour(), now.minute())),
        )
}

pub fn pilot_details(pilot: &PilotIntel, stats: &ZkillStats, _cx: &App) -> impl IntoElement {
    let id = pilot.character.id;
    div()
        .h(px(DETAILS_HEIGHT))
        .px_4()
        .py_3()
        .pl(px(56.))
        .bg(theme::color(BG_2))
        .border_t_1()
        .border_color(theme::color(BORDER))
        .overflow_hidden()
        .flex()
        .gap_6()
        .child(
            div()
                .flex_1()
                .child(heading("COMBAT"))
                .child(combat(stats, id)),
        )
        .child(
            div()
                .flex_1()
                .child(heading("SHIPS"))
                .child(ships(stats, id)),
        )
        .child(
            div()
                .flex_1()
                .child(heading("SYSTEMS"))
                .child(systems(stats, id)),
        )
        .child(
            div()
                .flex_1()
                .child(heading("ACTIVE HOURS"))
                .child(match &stats.activity {
                    Some(activity) => activity_heatmap(activity, id).into_any_element(),
                    None => empty_note("No activity data").into_any_element(),
                }),
        )
}
