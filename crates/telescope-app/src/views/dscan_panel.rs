use gpui_kit::component::Sizable as _;
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AnyElement, Context, Div, ElementId, FontWeight, Hsla, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, SharedString, StatefulInteractiveElement as _, Styled as _,
    Subscription, Window, div, img, px, relative,
};
use telescope_core::models::DscanEntry;
use telescope_core::view::dscan_view::{
    ClassIcon, Hull, TypeBucket, bar_width, bucket_by_type, class_icon, count_by_class,
    sort_classes_by_size, sort_types_by_size,
};
use telescope_core::view::format::ship_icon_url;

use crate::state::Stores;
use crate::theme::{self, BG_0, BG_1, BG_2, BG_3, BORDER, CYAN, RED, TEXT_1, TEXT_2, TEXT_3};
use crate::ui::{Icon, IconName, section_title};
use crate::views::motion;

pub struct DscanPanel {
    show_other: bool,
    /// Bumped for every new result so its bars grow in again.
    generation: u64,
    selected_class: Option<String>,
    _subscription: Subscription,
}

impl DscanPanel {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let scan = Stores::get(cx).scan;
        Self {
            show_other: false,
            generation: 0,
            selected_class: None,
            _subscription: cx.observe(&scan, |this, scan, cx| {
                // A new result invalidates the class filter.
                if scan.read(cx).dscan_loading() {
                    this.selected_class = None;
                    this.generation += 1;
                }
                cx.notify()
            }),
        }
    }
}

/// A background bar behind a list row, filling `percent` of its width.
fn bar(id: ElementId, percent: f64, color: Hsla) -> AnyElement {
    let fraction = (percent / 100.) as f32;
    motion::grow(
        id,
        div().absolute().top_0().bottom_0().left_0().bg(color),
        true,
        move |el, p| el.w(relative(fraction * p)),
    )
}

fn class_icon_name(icon: ClassIcon) -> IconName {
    match icon {
        ClassIcon::HeartPulse => IconName::HeartPulse,
        ClassIcon::CircleDot => IconName::CircleDot,
        ClassIcon::Zap => IconName::Zap,
        ClassIcon::Crosshair => IconName::Crosshair,
        ClassIcon::Eye => IconName::Eye,
        ClassIcon::Bomb => IconName::Bomb,
        ClassIcon::Satellite => IconName::Satellite,
        ClassIcon::Anchor => IconName::Anchor,
        ClassIcon::Truck => IconName::Truck,
        ClassIcon::Pickaxe => IconName::Pickaxe,
        ClassIcon::Swords => IconName::Swords,
        ClassIcon::ShieldHalf => IconName::ShieldHalf,
        ClassIcon::Shield => IconName::Shield,
        ClassIcon::Rocket => IconName::Rocket,
        ClassIcon::Ship => IconName::Ship,
    }
}

/// The in-game overview bracket for a ship class, or a Lucide icon for
/// classes without one.
fn class_glyph(class: &str, active: bool) -> AnyElement {
    match Hull::for_group(class) {
        Some(hull) => img(format!("brackets/{}.png", hull.bracket()))
            .size(px(16.))
            .flex_none()
            .opacity(if active { 1. } else { 0.6 })
            .into_any_element(),
        None => Icon::new(class_icon_name(class_icon(class)))
            .size_4()
            .text_color(theme::color(if active { CYAN } else { TEXT_3 }))
            .into_any_element(),
    }
}

fn type_icon(bucket: &TypeBucket, size: f32) -> impl IntoElement {
    match bucket.type_id {
        Some(id) => img(ship_icon_url(id, 64))
            .size(px(size))
            .flex_none()
            .rounded_sm()
            .into_any_element(),
        None => div()
            .size(px(size))
            .flex_none()
            .rounded_sm()
            .bg(theme::color(BG_3))
            .into_any_element(),
    }
}

fn headline(
    label: &'static str,
    value: usize,
    icon: Option<IconName>,
    accent: bool,
) -> impl IntoElement {
    div()
        .child(
            div()
                .flex()
                .items_center()
                .gap_1p5()
                .text_size(px(10.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme::color(TEXT_3))
                .when_some(icon, |el, icon| el.child(Icon::new(icon).size_3p5()))
                .child(label),
        )
        .child(
            div()
                .mt_1()
                .text_3xl()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme::color(if accent { CYAN } else { TEXT_1 }))
                .child(value.to_string()),
        )
}

fn empty_state() -> Div {
    div()
        .size_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_4()
        .text_color(theme::color(TEXT_3))
        .child(Icon::new(IconName::Radar).size(px(64.)).opacity(0.2))
        .child(
            div()
                .flex()
                .flex_col()
                .items_center()
                .child(
                    div()
                        .mb_1()
                        .text_sm()
                        .text_color(theme::color(TEXT_2))
                        .child("No D-scan results"),
                )
                .child(
                    div()
                        .text_xs()
                        .child("Paste directional scan output to begin"),
                ),
        )
}

impl Render for DscanPanel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let generation = self.generation;
        let scan = Stores::get(cx).scan.read(cx);
        let error = scan.dscan_error().map(str::to_string);
        let Some(result) = scan.dscan().cloned() else {
            return div()
                .size_full()
                .flex()
                .flex_col()
                .bg(theme::color(BG_0))
                .when_some(error, |el, e| el.child(error_banner(e)))
                .child(empty_state());
        };

        let ships: Vec<DscanEntry> = result
            .entries
            .iter()
            .filter(|e| e.is_ship)
            .cloned()
            .collect();
        let others: Vec<DscanEntry> = result
            .entries
            .iter()
            .filter(|e| !e.is_ship)
            .cloned()
            .collect();
        let mut types = bucket_by_type(&ships, |e| {
            e.group_name
                .clone()
                .unwrap_or_else(|| "Unknown class".into())
        });
        let mut classes = count_by_class(&ships);
        sort_types_by_size(&mut types);
        sort_classes_by_size(&mut classes);
        let other_types = bucket_by_type(&others, |e| {
            e.category_name.clone().unwrap_or_else(|| "Unknown".into())
        });
        let max_type = types.iter().map(|t| t.count).max().unwrap_or(0);
        let max_class = classes.iter().map(|c| c.count).max().unwrap_or(0);
        let selected = self.selected_class.clone();
        let visible_types: Vec<&TypeBucket> = types
            .iter()
            .filter(|t| selected.as_ref().is_none_or(|s| *s == t.subtitle))
            .collect();

        let ship_types =
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .overflow_hidden()
                .child(
                    div()
                        .flex()
                        .flex_none()
                        .items_center()
                        .gap_2()
                        .px_5()
                        .pt_5()
                        .pb_3()
                        .child(
                            Icon::new(IconName::Radar)
                                .size_4()
                                .text_color(theme::color(CYAN)),
                        )
                        .child(section_title("SHIP TYPES"))
                        .when_some(selected.clone(), |el, class| {
                            el.child(
                                div()
                                    .id("clear-class")
                                    .ml_1()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .pl_2()
                                    .pr_1p5()
                                    .py_0p5()
                                    .rounded_full()
                                    .border_1()
                                    .border_color(theme::tint(CYAN, 0x66))
                                    .bg(theme::tint(CYAN, 0x1a))
                                    .text_size(px(11.))
                                    .text_color(theme::color(CYAN))
                                    .cursor_pointer()
                                    .hover(|s| s.bg(theme::tint(CYAN, 0x33)))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.selected_class = None;
                                        cx.notify();
                                    }))
                                    .child(class)
                                    .child(Icon::new(IconName::X).size_3()),
                            )
                        }),
                )
                .child(
                    div()
                        .id("ship-types")
                        .flex_1()
                        .min_h_0()
                        .overflow_y_scroll()
                        .px_5()
                        .pb_4()
                        .when(types.is_empty(), |el| {
                            el.child(
                                div()
                                    .text_sm()
                                    .text_color(theme::color(TEXT_3))
                                    .child("No ships detected in this scan."),
                            )
                        })
                        .child(div().flex().flex_col().gap_1().children(
                            visible_types.into_iter().map(|t| {
                                div()
                                    .relative()
                                    .flex()
                                    .items_center()
                                    .gap_3()
                                    .overflow_hidden()
                                    .rounded_md()
                                    .bg(theme::color(BG_1))
                                    .px_3()
                                    .py_2()
                                    .child(bar(
                                        ElementId::NamedInteger(
                                            format!("type-bar-{}", t.type_name).into(),
                                            generation,
                                        ),
                                        bar_width(t.count, max_type, 6.),
                                        theme::tint(CYAN, 0x14),
                                    ))
                                    .child(type_icon(t, 32.))
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w_0()
                                            .child(
                                                div()
                                                    .truncate()
                                                    .text_sm()
                                                    .child(t.type_name.clone()),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap_1()
                                                    .text_xs()
                                                    .text_color(theme::color(TEXT_3))
                                                    .when_some(
                                                        Hull::for_group(&t.subtitle),
                                                        |el, hull| {
                                                            el.child(
                                                                img(format!(
                                                                    "brackets/{}.png",
                                                                    hull.bracket()
                                                                ))
                                                                .size(px(12.))
                                                                .flex_none()
                                                                .opacity(0.6),
                                                            )
                                                        },
                                                    )
                                                    .child(
                                                        div().truncate().child(t.subtitle.clone()),
                                                    ),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .flex_none()
                                            .text_xl()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(theme::color(CYAN))
                                            .child(t.count.to_string()),
                                    )
                            }),
                        ))
                        .when(self.show_other && !other_types.is_empty(), |el| {
                            el.child(
                                div()
                                    .mt_5()
                                    .mb_2()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        Icon::new(IconName::Boxes)
                                            .size_3p5()
                                            .text_color(theme::color(TEXT_3)),
                                    )
                                    .child(section_title("OTHER OBJECTS")),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .children(other_types.iter().map(|t| {
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_3()
                                            .px_3()
                                            .py_1p5()
                                            .text_color(theme::color(TEXT_2))
                                            .child(div().opacity(0.7).child(type_icon(t, 24.)))
                                            .child(
                                                div()
                                                    .flex_1()
                                                    .min_w_0()
                                                    .child(
                                                        div()
                                                            .truncate()
                                                            .text_sm()
                                                            .child(t.type_name.clone()),
                                                    )
                                                    .child(
                                                        div()
                                                            .truncate()
                                                            .text_xs()
                                                            .text_color(theme::color(TEXT_3))
                                                            .child(t.subtitle.clone()),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .flex_none()
                                                    .text_sm()
                                                    .font_weight(FontWeight::SEMIBOLD)
                                                    .child(t.count.to_string()),
                                            )
                                    })),
                            )
                        }),
                );

        let by_class =
            div()
                .w(px(360.))
                .flex_none()
                .flex()
                .flex_col()
                .overflow_hidden()
                .border_l_1()
                .border_color(theme::color(BORDER))
                .child(
                    div()
                        .flex_none()
                        .px_5()
                        .pt_5()
                        .pb_3()
                        .child(section_title("BY CLASS")),
                )
                .child(
                    div()
                        .id("by-class")
                        .flex_1()
                        .min_h_0()
                        .overflow_y_scroll()
                        .px_5()
                        .pb_4()
                        .when(classes.is_empty(), |el| {
                            el.child(div().text_sm().text_color(theme::color(TEXT_3)).child("—"))
                        })
                        .child(div().flex().flex_col().gap_1().children(classes.iter().map(
                            |class| {
                                let active = selected.as_deref() == Some(class.name.as_str());
                                let name = class.name.clone();
                                let fg = if active { CYAN } else { TEXT_1 };
                                div()
                                    .id(SharedString::from(format!("class-{}", class.name)))
                                    .relative()
                                    .flex()
                                    .items_center()
                                    .gap_2p5()
                                    .overflow_hidden()
                                    .rounded_md()
                                    .border_1()
                                    .px_2p5()
                                    .py_2()
                                    .cursor_pointer()
                                    .map(|el| {
                                        if active {
                                            el.border_color(theme::tint(CYAN, 0x80))
                                                .bg(theme::tint(CYAN, 0x1a))
                                        } else {
                                            el.border_color(gpui_kit::transparent_black()).hover(
                                                |s| {
                                                    s.border_color(theme::color(BORDER))
                                                        .bg(theme::color(BG_1))
                                                },
                                            )
                                        }
                                    })
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.selected_class = if this.selected_class.as_deref()
                                            == Some(name.as_str())
                                        {
                                            None
                                        } else {
                                            Some(name.clone())
                                        };
                                        cx.notify();
                                    }))
                                    .child(bar(
                                        ElementId::NamedInteger(
                                            format!("class-bar-{}", class.name).into(),
                                            generation,
                                        ),
                                        bar_width(class.count, max_class, 4.),
                                        if active {
                                            theme::tint(CYAN, 0x1a)
                                        } else {
                                            theme::color(BG_2)
                                        },
                                    ))
                                    .child(class_glyph(&class.name, active))
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w_0()
                                            .truncate()
                                            .text_sm()
                                            .text_color(theme::color(fg))
                                            .child(class.name.clone()),
                                    )
                                    .child(
                                        div()
                                            .flex_none()
                                            .text_sm()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(theme::color(if active {
                                                CYAN
                                            } else {
                                                TEXT_2
                                            }))
                                            .child(class.count.to_string()),
                                    )
                            },
                        ))),
                );

        div()
            .size_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(theme::color(BG_0))
            .when_some(error, |el, e| el.child(error_banner(e)))
            .child(
                div()
                    .flex()
                    .flex_none()
                    .items_end()
                    .gap_8()
                    .px_5()
                    .py_4()
                    .border_b_1()
                    .border_color(theme::color(BORDER))
                    .child(headline("SHIPS", ships.len(), Some(IconName::Ship), true))
                    .child(headline("SHIP TYPES", types.len(), None, false))
                    .child(headline("CLASSES", classes.len(), None, false)),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .overflow_hidden()
                    .child(ship_types)
                    .child(by_class),
            )
            .child(
                div()
                    .flex()
                    .flex_none()
                    .items_center()
                    .gap_2()
                    .px_5()
                    .py_3()
                    .border_t_1()
                    .border_color(theme::color(BORDER))
                    .text_xs()
                    .text_color(theme::color(TEXT_2))
                    .child(
                        Checkbox::new("show-other")
                            .small()
                            .checked(self.show_other)
                            .label(format!("Show other objects ({})", others.len()))
                            .on_change(cx.listener(|this, value: &bool, _, cx| {
                                this.show_other = *value;
                                cx.notify();
                            })),
                    )
                    .child(
                        div()
                            .text_color(theme::color(TEXT_3))
                            .child("— structures, drones, etc."),
                    ),
            )
    }
}

fn error_banner(error: String) -> impl IntoElement {
    div()
        .px_5()
        .py_3()
        .border_b_1()
        .border_color(theme::tint(RED, 0x33))
        .bg(theme::tint(RED, 0x14))
        .text_sm()
        .text_color(theme::color(RED))
        .child(error)
}
