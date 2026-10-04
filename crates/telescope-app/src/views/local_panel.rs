use std::collections::HashMap;
use std::rc::Rc;

use gpui_kit::component::{VirtualListScrollHandle, v_virtual_list};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AnyElement, Context, FontWeight, InteractiveElement as _, IntoElement, ParentElement as _,
    Pixels, Render, SharedString, Size, StatefulInteractiveElement as _, Styled as _, Subscription,
    Window, div, img, px, size,
};
use telescope_core::models::PilotIntel;
use telescope_core::view::format::{
    alliance_logo_url, character_portrait_url, corporation_logo_url, format_isk, format_ppk,
    kd_ratio, ship_icon_url, to_fixed,
};
use telescope_core::view::pilot_counts::tag_counts;
use telescope_core::view::pilot_filters::UNKNOWN_CORPORATION;
use telescope_core::view::pilot_sort::{SortDirection, SortKey, SortState, sort_pilots};
use telescope_core::view::pilot_tags::{
    DEFAULT_TAG_COLOR, DEFAULT_TAG_TEXT_COLOR, PilotTag, pilot_tags, pilot_tags_with_index,
};

use crate::state::Stores;
use crate::theme::{
    self, BG_1, BG_2, BG_3, BG_HOVER, BORDER, CYAN, CYAN_DIM, GREEN, ORANGE, RED, TEXT_1, TEXT_2,
    TEXT_3,
};
use crate::ui::{Icon, IconName, mono, section_title};
use crate::views::grid::{Col, cell, span};
use crate::views::intel_menu::{annotation_notes_button, pilot_context_menu};
use crate::views::pilot_details::{DETAILS_HEIGHT, pilot_details};

const ROW_HEIGHT: f32 = 40.;
const ERROR_HEIGHT: f32 = 24.;

const COLS: [Col; 12] = [
    Col::Fixed(64.),
    Col::Grow(180., 1.6),
    Col::Grow(110., 1.0),
    Col::Grow(140., 1.2),
    Col::Grow(140., 1.2),
    Col::Fixed(168.),
    Col::Fixed(56.),
    Col::Fixed(72.),
    Col::Fixed(84.),
    Col::Fixed(56.),
    Col::Fixed(56.),
    Col::Fixed(64.),
];

pub struct LocalPanel {
    sort: SortState,
    expanded: Option<i64>,
    rows: Vec<PilotIntel>,
    row_sizes: Rc<Vec<Size<Pixels>>>,
    scroll: VirtualListScrollHandle,
    _subscriptions: Vec<Subscription>,
}

impl LocalPanel {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let stores = Stores::get(cx);
        let settings = stores.settings.read(cx).get();
        let sort = SortState {
            key: SortKey::parse(&settings.sort_column).unwrap_or(SortKey::Threat),
            direction: SortDirection::parse(&settings.sort_direction)
                .unwrap_or(SortDirection::Desc),
        };
        let subscriptions = vec![
            cx.observe(&stores.scan, |this, _, cx| this.refresh(cx)),
            cx.observe(&stores.filters, |this, _, cx| this.refresh(cx)),
            cx.observe(&stores.intel, |this, _, cx| {
                let pilots = Stores::get(cx).scan.read(cx).pilots().to_vec();
                Stores::get(cx)
                    .filters
                    .update(cx, |filters, cx| filters.prune_tags(&pilots, cx));
                this.refresh(cx);
            }),
        ];
        let mut panel = Self {
            sort,
            expanded: None,
            rows: Vec::new(),
            row_sizes: Rc::new(Vec::new()),
            scroll: VirtualListScrollHandle::new(),
            _subscriptions: subscriptions,
        };
        panel.refresh(cx);
        panel
    }

    /// Re-filters and re-sorts the rows and recomputes their heights.
    fn refresh(&mut self, cx: &mut Context<Self>) {
        let stores = Stores::get(cx);
        let filters = stores.filters.read(cx);
        let mut rows: Vec<PilotIntel> = stores
            .scan
            .read(cx)
            .pilots()
            .iter()
            .filter(|p| filters.matches(p, cx))
            .cloned()
            .collect();
        sort_pilots(&mut rows, self.sort.key, self.sort.direction);
        self.row_sizes = Rc::new(
            rows.iter()
                .map(|p| size(px(1.), px(self.row_height(p))))
                .collect(),
        );
        self.rows = rows;
        cx.notify();
    }

    fn row_height(&self, pilot: &PilotIntel) -> f32 {
        let mut height = ROW_HEIGHT;
        if self.expanded == Some(pilot.character.id) && pilot.zkill.is_some() {
            height += DETAILS_HEIGHT;
        }
        if pilot.error.is_some() {
            height += ERROR_HEIGHT;
        }
        height
    }

    fn toggle_sort(&mut self, key: SortKey, cx: &mut Context<Self>) {
        self.sort.toggle(key);
        let sort = self.sort;
        Stores::get(cx).settings.update(cx, |settings, cx| {
            settings.update(cx, |s| {
                s.sort_column = sort.key.as_str().to_string();
                s.sort_direction = sort.direction.as_str().to_string();
            })
        });
        self.refresh(cx);
    }

    fn toggle_expand(&mut self, id: i64, cx: &mut Context<Self>) {
        self.expanded = if self.expanded == Some(id) {
            None
        } else {
            Some(id)
        };
        self.refresh(cx);
    }

    fn sort_header(
        &self,
        key: SortKey,
        label: impl IntoElement,
        right: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let active = self.sort.key == key;
        let chevron = match self.sort.direction {
            SortDirection::Asc => IconName::ChevronUp,
            SortDirection::Desc => IconName::ChevronDown,
        };
        div()
            .id(SharedString::from(format!("sort-{}", key.as_str())))
            .flex()
            .items_center()
            .gap_1()
            .when(right, |el| el.ml_auto())
            .cursor_pointer()
            .when(active, |el| el.text_color(theme::color(CYAN)))
            .hover(|s| s.text_color(theme::color(TEXT_1)))
            .child(label)
            .when(active, |el| el.child(Icon::new(chevron).size_3()))
            .on_click(cx.listener(move |this, _, _, cx| this.toggle_sort(key, cx)))
    }

    fn render_header(&self, count: usize, cx: &mut Context<Self>) -> impl IntoElement {
        let header_cell = |col: Col| cell(col).px_2().py_2();
        div()
            .flex()
            .flex_none()
            .bg(theme::color(BG_2))
            .border_b_1()
            .border_color(theme::color(BORDER))
            .text_size(px(10.))
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(theme::color(TEXT_3))
            .pl(px(3.))
            .child(header_cell(COLS[0]).child(self.sort_header(
                SortKey::Threat,
                "THREAT",
                false,
                cx,
            )))
            .child(
                header_cell(COLS[1]).child(
                    self.sort_header(
                        SortKey::Pilot,
                        div().flex().items_center().gap_2().child("PILOT").child(
                            div()
                                .px_1p5()
                                .rounded_sm()
                                .bg(theme::color(BG_3))
                                .font_family(mono())
                                .text_color(theme::color(CYAN))
                                .child(count.to_string()),
                        ),
                        false,
                        cx,
                    ),
                ),
            )
            .child(header_cell(COLS[2]).child("TAGS"))
            .child(header_cell(COLS[3]).child(self.sort_header(
                SortKey::Corporation,
                "CORPORATION",
                false,
                cx,
            )))
            .child(header_cell(COLS[4]).child(self.sort_header(
                SortKey::Alliance,
                "ALLIANCE",
                false,
                cx,
            )))
            .child(header_cell(COLS[5]).child("SHIPS"))
            .child(span(&COLS[6..8]).px_2().py_2().child(self.sort_header(
                SortKey::Kd,
                "K/D",
                true,
                cx,
            )))
            .child(header_cell(COLS[8]).child(self.sort_header(SortKey::Isk, "ISK", true, cx)))
            .child(header_cell(COLS[9]).child(self.sort_header(SortKey::Ppk, "PPK", true, cx)))
            .child(header_cell(COLS[10]).child(self.sort_header(SortKey::Cpk, "CPK", true, cx)))
            .child(header_cell(COLS[11]).child(self.sort_header(
                SortKey::Active,
                "ACTIVE",
                true,
                cx,
            )))
    }

    fn render_row(
        &self,
        pilot: &PilotIntel,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let id = pilot.character.id;
        let expanded = self.expanded == Some(id) && pilot.zkill.is_some();
        let intel = Stores::get(cx).intel.read(cx);
        let resolved = intel.resolve(pilot);
        let tags = pilot_tags(pilot, &resolved);
        let notes = annotation_notes_button(pilot, &resolved, cx);
        let threat = theme::threat_color(&pilot.threat_level);
        let row_cell = |col: Col| cell(col).px_2().py_1p5();
        let dash = || div().text_color(theme::color(TEXT_3)).child("—");
        let z = pilot.zkill.as_ref();

        let main_row = div()
            .id(("pilot-row", id as u64))
            .flex()
            .h(px(ROW_HEIGHT - 1.))
            .bg(theme::color(BG_1))
            .hover(|s| s.bg(theme::color(BG_HOVER)))
            .border_l(px(3.))
            .border_color(threat)
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, _, cx| this.toggle_expand(id, cx)))
            .child(row_cell(COLS[0]).child(threat_badge(&pilot.threat_level)))
            .child(
                row_cell(COLS[1]).child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .min_w_0()
                        .child(portrait(id))
                        .child(
                            div()
                                .truncate()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(pilot.character.name.clone()),
                        ),
                ),
            )
            .child(
                row_cell(COLS[2]).child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap_1()
                        .children(tags.iter().map(tag_badge))
                        .children(notes),
                ),
            )
            .child(
                row_cell(COLS[3]).child(affiliation(
                    pilot
                        .character
                        .corporation_id
                        .map(|id| corporation_logo_url(id, 64)),
                    pilot.character.corporation_name.clone(),
                )),
            )
            .child(
                row_cell(COLS[4]).child(affiliation(
                    pilot
                        .character
                        .alliance_id
                        .map(|id| alliance_logo_url(id, 64)),
                    pilot.character.alliance_name.clone(),
                )),
            )
            .child(
                row_cell(COLS[5]).child(match z.filter(|z| !z.top_ships.is_empty()) {
                    Some(z) => div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .children(z.top_ships.iter().take(5).map(|ship| {
                            let tooltip: SharedString =
                                format!("{} ({})", ship.ship_name, ship.kills).into();
                            div()
                                .id(("row-ship", (id as u64) << 20 | ship.ship_type_id as u64))
                                .size(px(26.))
                                .rounded_sm()
                                .overflow_hidden()
                                .bg(theme::color(BG_3))
                                .tooltip(move |window, cx| {
                                    gpui_kit::component::tooltip::Tooltip::new(tooltip.clone())
                                        .build(window, cx)
                                })
                                .child(img(ship_icon_url(ship.ship_type_id, 64)).size_full())
                        }))
                        .when(z.top_ships.len() > 5, |el| {
                            el.child(
                                div()
                                    .ml_0p5()
                                    .text_size(px(10.))
                                    .text_color(theme::color(TEXT_3))
                                    .child(format!("+{}", z.top_ships.len() - 5)),
                            )
                        })
                        .into_any_element(),
                    None => dash().into_any_element(),
                }),
            )
            .child(
                row_cell(COLS[6])
                    .justify_end()
                    .font_family(mono())
                    .text_sm()
                    .text_color(theme::color(TEXT_2))
                    .child(match z {
                        Some(z) => kd_ratio(z.ships_destroyed, z.ships_lost).into_any_element(),
                        None => dash().into_any_element(),
                    }),
            )
            .child(
                row_cell(COLS[7])
                    .font_family(mono())
                    .text_size(px(11.))
                    .child(match z {
                        Some(z) => plus_minus(
                            format!("+{}", group_thousands(z.ships_destroyed)),
                            format!("-{}", group_thousands(z.ships_lost)),
                        )
                        .into_any_element(),
                        None => dash().into_any_element(),
                    }),
            )
            .child(
                row_cell(COLS[8])
                    .font_family(mono())
                    .text_size(px(11.))
                    .child(match z {
                        Some(z) => plus_minus(
                            format!("+{}", format_isk(z.isk_destroyed)),
                            format!("-{}", format_isk(z.isk_lost)),
                        )
                        .into_any_element(),
                        None => dash().into_any_element(),
                    }),
            )
            .child(
                row_cell(COLS[9])
                    .justify_end()
                    .font_family(mono())
                    .text_xs()
                    .child(match z {
                        Some(z) => div()
                            .text_color(theme::color(TEXT_2))
                            .child(format_ppk(z.points_destroyed, z.ships_destroyed))
                            .into_any_element(),
                        None => dash().into_any_element(),
                    }),
            )
            .child(
                row_cell(COLS[10])
                    .justify_end()
                    .font_family(mono())
                    .text_xs()
                    .child(match z {
                        Some(z) => div()
                            .text_color(theme::color(TEXT_2))
                            .child(to_fixed(z.avg_attackers, 1))
                            .into_any_element(),
                        None => dash().into_any_element(),
                    }),
            )
            .child(
                row_cell(COLS[11])
                    .justify_end()
                    .font_family(mono())
                    .child(match z {
                        Some(z) => {
                            let hot = z.active_pvp_kills > 20;
                            div()
                                .text_sm()
                                .map(|el| {
                                    if hot {
                                        el.text_color(theme::color(ORANGE))
                                            .font_weight(FontWeight::SEMIBOLD)
                                    } else {
                                        el.text_color(theme::color(TEXT_2))
                                    }
                                })
                                .child(z.active_pvp_kills.to_string())
                                .into_any_element()
                        }
                        None => dash().into_any_element(),
                    }),
            );

        let main_row = pilot_context_menu(main_row, pilot, window, cx);

        div()
            .w_full()
            .h(px(self.row_height(pilot)))
            .flex()
            .flex_col()
            .border_b_1()
            .border_color(theme::color(BORDER))
            .child(main_row)
            .when_some(z.filter(|_| expanded), |el, z| {
                el.child(pilot_details(pilot, z, cx))
            })
            .when_some(pilot.error.clone(), |el, error| {
                el.child(
                    div()
                        .h(px(ERROR_HEIGHT))
                        .flex()
                        .items_center()
                        .px_4()
                        .pl(px(56.))
                        .text_xs()
                        .text_color(theme::color(RED))
                        .bg(theme::tint(RED, 0x0d))
                        .child(error),
                )
            })
            .into_any_element()
    }

    fn render_table(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .overflow_hidden()
            .child(self.render_header(self.rows.len(), cx))
            .child(
                v_virtual_list(
                    cx.entity(),
                    "pilot-rows",
                    self.row_sizes.clone(),
                    |this, range, window, cx| {
                        let rows: Vec<PilotIntel> = this.rows[range].to_vec();
                        rows.iter()
                            .map(|pilot| this.render_row(pilot, window, cx))
                            .collect()
                    },
                )
                .track_scroll(&self.scroll)
                .flex_1(),
            )
    }

    fn render_sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let stores = Stores::get(cx);
        let pilots = stores.scan.read(cx).pilots();
        let intel = stores.intel.read(cx);
        let index = intel.annotation_index();
        let filters = stores.filters.read(cx).get().clone();
        let tags = tag_counts(pilots, |p| pilot_tags_with_index(p, index));
        let corps = groups(pilots, |p| {
            let name = p
                .character
                .corporation_name
                .clone()
                .filter(|n| !n.is_empty())
                .unwrap_or_else(|| UNKNOWN_CORPORATION.to_string());
            Some((
                name,
                p.character.corporation_id.unwrap_or(0),
                p.character.corporation_ticker.clone().unwrap_or_default(),
            ))
        });
        let alliances = groups(pilots, |p| {
            let name = p
                .character
                .alliance_name
                .clone()
                .filter(|n| !n.is_empty())?;
            let id = p.character.alliance_id.filter(|id| *id != 0)?;
            Some((
                name,
                id,
                p.character.alliance_ticker.clone().unwrap_or_default(),
            ))
        });
        let has_filters = filters.has_active_filters();

        div()
            .w(px(256.))
            .h_full()
            .flex()
            .flex_col()
            .flex_none()
            .bg(theme::color(BG_1))
            .border_l_1()
            .border_color(theme::color(BORDER))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .px_4()
                    .py_3()
                    .border_b_1()
                    .border_color(theme::color(BORDER))
                    .child(section_title("FILTERS"))
                    .child(
                        div()
                            .id("clear-filters")
                            .size(px(20.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_sm()
                            .text_color(theme::color(TEXT_3))
                            .when(!has_filters, |el| el.invisible())
                            .cursor_pointer()
                            .hover(|s| {
                                s.bg(theme::color(BG_HOVER))
                                    .text_color(theme::color(TEXT_1))
                            })
                            .on_click(|_, _, cx| {
                                Stores::get(cx)
                                    .filters
                                    .update(cx, |f, cx| f.update(cx, |f| f.clear()))
                            })
                            .child(Icon::new(IconName::X).size_3()),
                    ),
            )
            .child(
                div()
                    .id("filter-lists")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .when(!tags.is_empty(), |el| {
                        el.child(
                            div()
                                .p_3()
                                .border_b_1()
                                .border_color(theme::color(BORDER))
                                .child(section_title("TAGS").mb_2())
                                .child(div().flex().flex_wrap().gap_1p5().children(
                                    tags.into_iter().map(|t| {
                                        let selected = filters.selected_tags.contains(&t.tag);
                                        let color = t
                                            .color
                                            .clone()
                                            .unwrap_or_else(|| DEFAULT_TAG_COLOR.into());
                                        let text = t
                                            .color
                                            .clone()
                                            .unwrap_or_else(|| DEFAULT_TAG_TEXT_COLOR.into());
                                        let tag = t.tag.clone();
                                        div()
                                            .id(SharedString::from(format!("tag-filter-{}", t.tag)))
                                            .flex()
                                            .items_center()
                                            .gap_1p5()
                                            .px_2()
                                            .py_1()
                                            .rounded_sm()
                                            .border_1()
                                            .text_size(px(10.))
                                            .font_weight(FontWeight::BOLD)
                                            .bg(theme::hex_tint(&color, 0x22, TEXT_3))
                                            .text_color(theme::hex_or(&text, TEXT_2))
                                            .map(|el| {
                                                if selected {
                                                    el.border_color(theme::hex_or(&color, TEXT_3))
                                                } else {
                                                    el.border_color(gpui_kit::transparent_black())
                                                        .opacity(0.7)
                                                        .hover(|s| s.opacity(1.))
                                                }
                                            })
                                            .cursor_pointer()
                                            .on_click(move |_, _, cx| {
                                                Stores::get(cx).filters.update(cx, |f, cx| {
                                                    f.update(cx, |f| f.toggle_tag(&tag))
                                                })
                                            })
                                            .child(t.tag)
                                            .child(
                                                div()
                                                    .font_family(mono())
                                                    .text_size(px(9.))
                                                    .opacity(0.75)
                                                    .child(t.count.to_string()),
                                            )
                                    }),
                                )),
                        )
                    })
                    .when(!alliances.is_empty(), |el| {
                        el.child(
                            div()
                                .p_3()
                                .border_b_1()
                                .border_color(theme::color(BORDER))
                                .child(filter_group(
                                    "ALLIANCES",
                                    alliances,
                                    &filters.selected_alliances,
                                    alliance_logo_url,
                                    |f, name| f.toggle_alliance(name),
                                )),
                        )
                    })
                    .child(div().p_3().child(filter_group(
                        "CORPORATIONS",
                        corps,
                        &filters.selected_corps,
                        corporation_logo_url,
                        |f, name| f.toggle_corp(name),
                    ))),
            )
    }
}

impl Render for LocalPanel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let scan = Stores::get(cx).scan.read(cx);
        if scan.pilots().is_empty() {
            let loading = scan.loading();
            return div()
                .size_full()
                .when(!loading, |el| el.child(empty_state()))
                .into_any_element();
        }
        div()
            .size_full()
            .flex()
            .overflow_hidden()
            .child(self.render_table(cx))
            .child(self.render_sidebar(cx))
            .into_any_element()
    }
}

struct Group {
    name: String,
    id: i64,
    ticker: String,
    count: usize,
}

fn groups(
    pilots: &[PilotIntel],
    key: impl Fn(&PilotIntel) -> Option<(String, i64, String)>,
) -> Vec<Group> {
    let mut order: Vec<Group> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();
    for (name, id, ticker) in pilots.iter().filter_map(key) {
        match index.get(&name) {
            Some(&i) => order[i].count += 1,
            None => {
                index.insert(name.clone(), order.len());
                order.push(Group {
                    name,
                    id,
                    ticker,
                    count: 1,
                });
            }
        }
    }
    order.sort_by(|a, b| b.count.cmp(&a.count));
    order
}

fn filter_group(
    title: &'static str,
    items: Vec<Group>,
    selected: &std::collections::BTreeSet<String>,
    logo: fn(i64, u32) -> String,
    toggle: fn(&mut telescope_core::view::pilot_filters::PilotFilters, &str),
) -> impl IntoElement {
    let logo = move |id| logo(id, 64);
    div()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .mb_2()
                .child(section_title(title))
                .child(
                    div()
                        .px_1p5()
                        .rounded_sm()
                        .bg(theme::color(BG_3))
                        .font_family(mono())
                        .text_size(px(9.))
                        .text_color(theme::color(TEXT_3))
                        .child(items.len().to_string()),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap_0p5()
                .children(items.into_iter().map(|item| {
                    let active = selected.contains(&item.name);
                    let name = item.name.clone();
                    div()
                        .id(SharedString::from(format!("{title}-{}", item.name)))
                        .flex()
                        .items_center()
                        .gap_2()
                        .px_2()
                        .py_1p5()
                        .rounded_sm()
                        .border_1()
                        .text_size(px(11.))
                        .cursor_pointer()
                        .map(|el| {
                            if active {
                                el.bg(theme::tint(CYAN, 0x1a))
                                    .border_color(theme::color(CYAN_DIM))
                                    .text_color(theme::color(CYAN))
                            } else {
                                el.border_color(gpui_kit::transparent_black())
                                    .text_color(theme::color(TEXT_1))
                                    .hover(|s| s.bg(theme::color(BG_HOVER)))
                            }
                        })
                        .on_click(move |_, _, cx| {
                            Stores::get(cx)
                                .filters
                                .update(cx, |f, cx| f.update(cx, |f| toggle(f, &name)))
                        })
                        .when(item.id != 0, |el| {
                            el.child(
                                img(logo(item.id))
                                    .size(px(22.))
                                    .rounded_sm()
                                    .bg(theme::color(BG_3))
                                    .flex_none(),
                            )
                        })
                        .when(!item.ticker.is_empty(), |el| {
                            el.child(
                                div()
                                    .flex_none()
                                    .font_family(mono())
                                    .text_size(px(10.))
                                    .text_color(theme::color(if active { CYAN } else { TEXT_2 }))
                                    .child(format!("[{}]", item.ticker)),
                            )
                        })
                        .child(div().flex_1().min_w_0().truncate().child(item.name))
                        .child(
                            div()
                                .flex_none()
                                .px_1p5()
                                .rounded_sm()
                                .font_family(mono())
                                .text_size(px(10.))
                                .map(|el| {
                                    if active {
                                        el.bg(theme::tint(CYAN, 0x33))
                                            .text_color(theme::color(CYAN))
                                    } else {
                                        el.bg(theme::color(BG_3)).text_color(theme::color(TEXT_3))
                                    }
                                })
                                .child(item.count.to_string()),
                        )
                })),
        )
}

fn empty_state() -> impl IntoElement {
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
                        .text_sm()
                        .mb_1()
                        .text_color(theme::color(TEXT_2))
                        .child("No pilots scanned"),
                )
                .child(
                    div()
                        .text_xs()
                        .child("Paste names from local chat to begin"),
                ),
        )
}

pub fn portrait(character_id: i64) -> AnyElement {
    if character_id == 0 {
        return div()
            .size(px(26.))
            .flex_none()
            .rounded_sm()
            .bg(theme::color(BG_3))
            .flex()
            .items_center()
            .justify_center()
            .text_xs()
            .text_color(theme::color(TEXT_3))
            .child("?")
            .into_any_element();
    }
    img(character_portrait_url(character_id, 64))
        .size(px(26.))
        .flex_none()
        .rounded_sm()
        .bg(theme::color(BG_3))
        .into_any_element()
}

pub fn threat_badge(level: &str) -> impl IntoElement {
    let color = theme::threat_color(level);
    let text: String = level.chars().take(3).collect::<String>().to_uppercase();
    div()
        .px_1p5()
        .py_0p5()
        .rounded_sm()
        .border_1()
        .border_color(color.opacity(0.4))
        .bg(color.opacity(0.12))
        .text_color(color)
        .text_size(px(10.))
        .font_weight(FontWeight::BOLD)
        .child(text)
}

pub fn tag_badge(tag: &PilotTag) -> impl IntoElement {
    let color = tag.color.as_deref().unwrap_or(DEFAULT_TAG_COLOR);
    let text = tag.color.as_deref().unwrap_or(DEFAULT_TAG_TEXT_COLOR);
    div()
        .px_1p5()
        .rounded_sm()
        .text_size(px(10.))
        .font_weight(FontWeight::SEMIBOLD)
        .bg(theme::hex_tint(color, 0x22, TEXT_3))
        .text_color(theme::hex_or(text, TEXT_2))
        .child(tag.text.clone())
}

fn affiliation(logo: Option<String>, name: Option<String>) -> impl IntoElement {
    let name = name.filter(|n| !n.is_empty());
    div()
        .flex()
        .items_center()
        .gap_1p5()
        .min_w_0()
        .text_xs()
        .text_color(theme::color(TEXT_2))
        .when_some(logo, |el, logo| {
            el.child(img(logo).size(px(16.)).rounded_sm().flex_none())
        })
        .child(div().truncate().child(name.unwrap_or_else(|| "—".into())))
}

fn plus_minus(plus: String, minus: String) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .line_height(px(14.))
        .child(div().text_color(theme::color(GREEN)).child(plus))
        .child(div().text_color(theme::color(RED)).child(minus))
}

fn group_thousands(n: i64) -> String {
    let digits = n.unsigned_abs().to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    if n < 0 { format!("-{out}") } else { out }
}
