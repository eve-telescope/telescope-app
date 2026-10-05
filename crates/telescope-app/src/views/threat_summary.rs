use gpui_kit::{
    IntoElement, ParentElement as _, Styled as _, div, prelude::FluentBuilder as _, px,
};
use telescope_core::models::PilotIntel;

use crate::theme::{self, TEXT_2, TEXT_3};
use crate::ui::{Icon, IconName, dot, mono};

#[derive(Default)]
struct Counts {
    extreme: usize,
    high: usize,
    moderate: usize,
    low: usize,
    rest: usize,
}

fn count(pilots: &[PilotIntel]) -> Counts {
    let mut counts = Counts::default();
    for pilot in pilots {
        match pilot.threat_level.to_ascii_lowercase().as_str() {
            "extreme" => counts.extreme += 1,
            "high" => counts.high += 1,
            "moderate" => counts.moderate += 1,
            "low" => counts.low += 1,
            _ => counts.rest += 1,
        }
    }
    counts
}

fn row(
    level: &str,
    label: &str,
    value: usize,
    value_color: gpui_kit::Hsla,
    bold: bool,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .text_xs()
        .child(
            div()
                .flex()
                .items_center()
                .gap_1p5()
                .child(dot(theme::threat_color(level), 8.))
                .child(
                    div()
                        .text_color(theme::color(TEXT_2))
                        .child(label.to_string()),
                ),
        )
        .child(
            div()
                .font_family(mono())
                .text_color(value_color)
                .when(bold, |el| el.font_weight(gpui_kit::FontWeight::SEMIBOLD))
                .child(value.to_string()),
        )
}

pub fn threat_summary(pilots: &[PilotIntel]) -> impl IntoElement {
    let c = count(pilots);
    let dangerous = c.extreme + c.high;
    let extreme = theme::threat_color("extreme");
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .when(c.extreme > 0, |el| {
                    el.child(row("extreme", "Extreme", c.extreme, extreme, true))
                })
                .when(c.high > 0, |el| {
                    el.child(row(
                        "high",
                        "High",
                        c.high,
                        theme::threat_color("high"),
                        true,
                    ))
                })
                .when(c.moderate > 0, |el| {
                    el.child(row(
                        "moderate",
                        "Moderate",
                        c.moderate,
                        theme::threat_color("moderate"),
                        false,
                    ))
                })
                .when(c.low > 0, |el| {
                    el.child(row("low", "Low", c.low, theme::threat_color("low"), false))
                })
                .when(c.rest > 0, |el| {
                    el.child(row(
                        "minimal",
                        "Minimal/Unknown",
                        c.rest,
                        theme::color(TEXT_3),
                        false,
                    ))
                }),
        )
        .when(dangerous > 0, |el| {
            el.child(
                div()
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .px_2()
                    .py_1p5()
                    .rounded_sm()
                    .border_1()
                    .border_color(extreme.opacity(0.3))
                    .bg(extreme.opacity(0.1))
                    .text_size(px(10.))
                    .text_color(extreme)
                    .child(Icon::new(IconName::TriangleAlert).size_3())
                    .child(format!(
                        "{} dangerous pilot{} detected",
                        dangerous,
                        if dangerous > 1 { "s" } else { "" }
                    )),
            )
        })
}
