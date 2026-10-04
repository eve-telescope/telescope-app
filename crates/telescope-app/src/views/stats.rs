//! Single-line kill and ISK cells. Each shows one number at a glance and puts
//! the exact figures in a tooltip, instead of stacking +/- lines in the row.

use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::{
    AnyElement, ElementId, InteractiveElement as _, IntoElement, ParentElement as _, Pixels,
    SharedString, StatefulInteractiveElement as _, Styled as _, div, px, relative,
};
use telescope_core::models::ZkillStats;
use telescope_core::view::format::{format_isk, kd_ratio};

use crate::theme::{self, BG_3, GREEN, ORANGE, RED, TEXT_1, TEXT_2};
use crate::ui::mono;

pub fn group_thousands(n: i64) -> String {
    let digits = n.unsigned_abs().to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    if n < 0 { format!("-{out}") } else { out }
}

/// Share of ISK destroyed out of all ISK involved, or `None` with no history.
pub fn isk_efficiency(z: &ZkillStats) -> Option<f64> {
    let total = z.isk_destroyed + z.isk_lost;
    (total > 0.).then(|| z.isk_destroyed / total)
}

fn with_tooltip(id: ElementId, text: String) -> gpui_kit::Stateful<gpui_kit::Div> {
    let text: SharedString = text.into();
    div()
        .id(id)
        .tooltip(move |window, cx| Tooltip::new(text.clone()).build(window, cx))
}

/// The K/D ratio next to a bar split green/red by kills and losses.
pub fn kd_cell(id: ElementId, z: &ZkillStats, bar_width: f32, text_size: Pixels) -> AnyElement {
    let total = z.ships_destroyed + z.ships_lost;
    let kill_share = if total > 0 {
        z.ships_destroyed as f32 / total as f32
    } else {
        0.
    };
    with_tooltip(
        id,
        format!(
            "{} kills · {} losses",
            group_thousands(z.ships_destroyed),
            group_thousands(z.ships_lost)
        ),
    )
    .w_full()
    .flex()
    .items_center()
    .justify_end()
    .gap_2()
    .child(
        div()
            .font_family(mono())
            .text_size(text_size)
            .text_color(theme::color(if kill_share >= 0.5 {
                TEXT_1
            } else {
                TEXT_2
            }))
            .child(kd_ratio(z.ships_destroyed, z.ships_lost)),
    )
    .child(
        div()
            .w(px(bar_width))
            .h(px(4.))
            .flex_none()
            .rounded_full()
            .overflow_hidden()
            .bg(if total > 0 {
                theme::tint(RED, 0xb3)
            } else {
                theme::color(BG_3)
            })
            .child(
                div()
                    .h_full()
                    .w(relative(kill_share))
                    .bg(theme::color(GREEN)),
            ),
    )
    .into_any_element()
}

/// ISK destroyed, colored by ISK efficiency.
pub fn isk_cell(id: ElementId, z: &ZkillStats, text_size: Pixels) -> AnyElement {
    let efficiency = isk_efficiency(z);
    let color = match efficiency {
        Some(e) if e >= 0.7 => GREEN,
        Some(e) if e >= 0.4 => ORANGE,
        Some(_) => RED,
        None => TEXT_2,
    };
    let tooltip = match efficiency {
        Some(e) => format!(
            "{} destroyed · {} lost · {:.0}% efficient",
            format_isk(z.isk_destroyed),
            format_isk(z.isk_lost),
            e * 100.
        ),
        None => "No ISK history".to_string(),
    };
    with_tooltip(id, tooltip)
        .w_full()
        .flex()
        .justify_end()
        .font_family(mono())
        .text_size(text_size)
        .text_color(theme::color(color))
        .child(format_isk(z.isk_destroyed))
        .into_any_element()
}
