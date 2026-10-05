//! Single-line kill and ISK cells. Each shows one number at a glance and puts
//! the exact figures in a tooltip, instead of stacking +/- lines in the row.

use gpui_kit::{
    AnyElement, AppContext as _, Context, Div, ElementId, FontWeight, Hsla,
    InteractiveElement as _, IntoElement, ParentElement as _, Pixels, Render,
    StatefulInteractiveElement as _, Styled as _, Window, div, px, relative,
};
use telescope_core::domain::threat::{DangerScore, danger_score};
use telescope_core::models::{PilotIntel, ZkillStats};
use telescope_core::view::format::{format_isk, format_ppk, kd_ratio};

use crate::theme::{self, BG_3, CYAN, GREEN, ORANGE, SURFACE, TEXT_1, TEXT_2, TEXT_3};
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

/// One color per score part, in the order `danger_score` lists them.
const PART_COLORS: [u32; 6] = [CYAN, 0x9d8cff, GREEN, 0xf5c542, ORANGE, 0xff6fae];

fn part_color(ix: usize) -> Hsla {
    theme::color(PART_COLORS[ix % PART_COLORS.len()])
}

fn caption(text: &'static str) -> Div {
    div()
        .text_size(px(10.))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(theme::color(TEXT_3))
        .child(text)
}

fn stat(label: &'static str, value: String, detail: String) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_0p5()
        .child(caption(label))
        .child(
            div()
                .font_family(mono())
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme::color(TEXT_1))
                .child(value),
        )
        .child(
            div()
                .text_size(px(10.))
                .text_color(theme::color(TEXT_3))
                .child(detail),
        )
}

/// The hover card behind a danger score: the total, what each part adds,
/// and the killboard numbers the score replaced.
struct DangerCard {
    z: ZkillStats,
    score: DangerScore,
    level: String,
}

impl Render for DangerCard {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let (z, score) = (&self.z, &self.score);
        let color = theme::threat_color(&self.level);
        let efficiency =
            isk_efficiency(z).map_or("—".to_string(), |e| format!("{:.0}%", e * 100.));

        let header = div()
            .flex()
            .items_end()
            .justify_between()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .child(caption("DANGER SCORE"))
                    .child(
                        div()
                            .flex()
                            .items_baseline()
                            .gap_1()
                            .child(
                                div()
                                    .text_size(px(28.))
                                    .line_height(px(30.))
                                    .font_family(mono())
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(color)
                                    .child(score.total.to_string()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::color(TEXT_3))
                                    .child("/ 100"),
                            ),
                    ),
            )
            .child(
                div()
                    .mb_1()
                    .px_2()
                    .py_0p5()
                    .rounded_sm()
                    .bg(color.opacity(0.14))
                    .text_size(px(10.))
                    .font_weight(FontWeight::BOLD)
                    .text_color(color)
                    .child(self.level.to_uppercase()),
            );

        let stacked = div()
            .h(px(6.))
            .flex()
            .gap(px(2.))
            .rounded_full()
            .overflow_hidden()
            .bg(theme::color(BG_3))
            .children(score.parts.iter().enumerate().map(|(ix, part)| {
                div()
                    .h_full()
                    .flex_none()
                    .w(relative((part.weight * part.value) as f32))
                    .bg(part_color(ix))
            }));

        let parts = div()
            .flex()
            .flex_col()
            .gap_1p5()
            .child(caption("BREAKDOWN"))
            .children(score.parts.iter().enumerate().map(|(ix, part)| {
                let max = (part.weight * 100.).round();
                let points = (part.weight * part.value * 100.).round();
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .text_xs()
                    .child(
                        div()
                            .size(px(6.))
                            .flex_none()
                            .rounded_full()
                            .bg(part_color(ix)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_color(theme::color(TEXT_2))
                            .child(part.label),
                    )
                    .child(
                        div()
                            .w(px(56.))
                            .h(px(3.))
                            .rounded_full()
                            .bg(theme::color(BG_3))
                            .child(
                                div()
                                    .h_full()
                                    .rounded_full()
                                    .w(relative(part.value as f32))
                                    .bg(part_color(ix)),
                            ),
                    )
                    .child(
                        div()
                            .w(px(40.))
                            .flex()
                            .justify_end()
                            .font_family(mono())
                            .child(
                                div()
                                    .text_color(theme::color(TEXT_1))
                                    .child(format!("{points:.0}")),
                            )
                            .child(
                                div()
                                    .text_color(theme::color(TEXT_3))
                                    .child(format!("/{max:.0}")),
                            ),
                    )
            }));

        let record = div()
            .flex()
            .flex_col()
            .gap_2()
            .child(caption("RECORD"))
            .child(
                div()
                    .grid()
                    .grid_cols(2)
                    .gap_x_4()
                    .gap_y_3()
                    .child(stat(
                        "K/D",
                        kd_ratio(z.ships_destroyed, z.ships_lost),
                        format!(
                            "{} kills · {} losses",
                            group_thousands(z.ships_destroyed),
                            group_thousands(z.ships_lost)
                        ),
                    ))
                    .child(stat(
                        "ISK DESTROYED",
                        format_isk(z.isk_destroyed),
                        format!("{efficiency} efficiency"),
                    ))
                    .child(stat(
                        "AVG GANG",
                        format!("{:.1}", z.avg_attackers),
                        format!(
                            "{} points per kill",
                            format_ppk(z.points_destroyed, z.ships_destroyed)
                        ),
                    ))
                    .child(stat(
                        "RECENT KILLS",
                        group_thousands(z.recent_kills),
                        format!("last 3 months · {} this week", z.active_pvp_kills),
                    )),
            );

        div().pl_3().pt_3().child(
            div()
                .w(px(300.))
                .flex()
                .flex_col()
                .gap_3()
                .p_3()
                .rounded_lg()
                .bg(theme::color(SURFACE))
                .border_1()
                .border_color(theme::hairline(0x14))
                .shadow_lg()
                .text_color(theme::color(TEXT_1))
                .child(header)
                .child(stacked)
                .child(div().h(px(1.)).bg(theme::hairline(0x0c)))
                .child(parts)
                .child(div().h(px(1.)).bg(theme::hairline(0x0c)))
                .child(record),
        )
    }
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
    let level = pilot.threat_level.clone();
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
        .tooltip(move |_, cx| {
            let card = DangerCard {
                z: z.clone(),
                score: score.clone(),
                level: level.clone(),
            };
            cx.new(|_| card).into()
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
