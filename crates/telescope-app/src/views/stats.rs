//! Single-line kill and ISK cells. Each shows one number at a glance and puts
//! the exact figures in a tooltip, instead of stacking +/- lines in the row.

use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::{
    AnyElement, ElementId, InteractiveElement as _, IntoElement, ParentElement as _, Pixels,
    StatefulInteractiveElement as _, Styled as _, div, px, relative,
};
use telescope_core::domain::threat::{DangerScore, danger_score};
use telescope_core::models::{PilotIntel, ZkillStats};
use telescope_core::view::format::{format_isk, format_ppk, kd_ratio};

use crate::theme::{self, BG_3, CYAN, TEXT_2};
use crate::ui::mono;
use crate::views::motion;

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

fn breakdown(z: &ZkillStats, score: &DangerScore) -> AnyElement {
    let row = |label: String, value: String| {
        div()
            .flex()
            .justify_between()
            .gap_4()
            .text_xs()
            .child(div().text_color(theme::color(TEXT_2)).child(label))
            .child(div().font_family(mono()).child(value))
    };
    let efficiency = isk_efficiency(z).map_or("—".to_string(), |e| format!("{:.0}%", e * 100.));
    div()
        .w(px(260.))
        .flex()
        .flex_col()
        .gap_1()
        .children(score.parts.iter().map(|part| {
            div()
                .flex()
                .items_center()
                .gap_2()
                .text_xs()
                .child(
                    div()
                        .flex_1()
                        .text_color(theme::color(TEXT_2))
                        .child(part.label),
                )
                .child(
                    div()
                        .w(px(60.))
                        .h(px(4.))
                        .rounded_full()
                        .bg(theme::color(BG_3))
                        .child(
                            div()
                                .h_full()
                                .rounded_full()
                                .w(relative(part.value as f32))
                                .bg(theme::color(CYAN)),
                        ),
                )
                .child(
                    div()
                        .w(px(28.))
                        .text_right()
                        .font_family(mono())
                        .child(format!("{:.0}", part.weight * part.value * 100.)),
                )
        }))
        .child(div().h(px(1.)).my_1().bg(theme::color(BG_3)))
        .child(row(
            "K/D".into(),
            format!(
                "{} ({} / {})",
                kd_ratio(z.ships_destroyed, z.ships_lost),
                group_thousands(z.ships_destroyed),
                group_thousands(z.ships_lost)
            ),
        ))
        .child(row(
            "ISK".into(),
            format!("{} · {efficiency}", format_isk(z.isk_destroyed)),
        ))
        .child(row(
            "CPK · PPK".into(),
            format!(
                "{:.1} · {}",
                z.avg_attackers,
                format_ppk(z.points_destroyed, z.ships_destroyed)
            ),
        ))
        .child(row(
            "Kills 3 months · 7 days".into(),
            format!("{} · {}", z.recent_kills, z.active_pvp_kills),
        ))
        .into_any_element()
}

/// The danger score with a bar in its threat color; hovering shows how the
/// score is made up and the raw numbers it replaced.
pub fn danger_cell(
    id: ElementId,
    pilot: &PilotIntel,
    bar_width: f32,
    text_size: Pixels,
    fresh: bool,
) -> AnyElement {
    let Some(z) = pilot.zkill.clone() else {
        return div()
            .text_color(theme::color(TEXT_2))
            .child("—")
            .into_any_element();
    };
    let Some(score) = danger_score(&z) else {
        return div()
            .text_color(theme::color(TEXT_2))
            .child("—")
            .into_any_element();
    };
    let color = theme::threat_color(&pilot.threat_level);
    let total = score.total;
    let fill = motion::grow(
        (id.clone(), "fill"),
        div().h_full().rounded_full().bg(color),
        fresh,
        move |el, p| el.w(relative(total as f32 / 100. * p)),
    );
    let number = motion::grow(
        (id.clone(), "number"),
        div()
            .w(px(24.))
            .text_right()
            .font_family(mono())
            .text_size(text_size)
            .font_weight(gpui_kit::FontWeight::SEMIBOLD)
            .text_color(color),
        fresh,
        move |el, p| el.child(((total as f32 * p).round() as u8).to_string()),
    );
    div()
        .id(id)
        .w_full()
        .flex()
        .items_center()
        .justify_end()
        .gap_2()
        .tooltip(move |window, cx| {
            let (z, score) = (z.clone(), score.clone());
            Tooltip::element(move |_, _| breakdown(&z, &score)).build(window, cx)
        })
        .child(
            div()
                .w(px(bar_width))
                .h(px(4.))
                .flex_none()
                .rounded_full()
                .bg(theme::color(BG_3))
                .child(fill),
        )
        .child(number)
        .into_any_element()
}
