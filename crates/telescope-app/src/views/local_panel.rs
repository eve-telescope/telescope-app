use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

use gpui_kit::component::{VirtualListScrollHandle, v_virtual_list};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AnyElement, Context, Div, FocusHandle, FontWeight, InteractiveElement as _, IntoElement,
    KeyDownEvent, MouseButton, MouseDownEvent, ParentElement as _, Pixels, Point, Render,
    SharedString, Size, Stateful, StatefulInteractiveElement as _, Styled as _, Subscription,
    Window, anchored, deferred, div, img, px, size,
};
use telescope_core::models::PilotIntel;
use telescope_core::view::format::{
    alliance_logo_url, character_portrait_url, corporation_logo_url, ship_icon_url,
};
use telescope_core::view::pilot_counts::tag_counts;
use telescope_core::view::pilot_filters::UNKNOWN_CORPORATION;
use telescope_core::view::pilot_sort::{SortDirection, SortKey, SortState, sort_pilots};
use telescope_core::view::pilot_tags::{
    DEFAULT_TAG_COLOR, DEFAULT_TAG_TEXT_COLOR, pilot_tags, pilot_tags_with_index,
};

use crate::state::Stores;
use crate::theme::{
    self, BG_0, BG_1, BG_2, BG_3, BG_HOVER, BORDER, CYAN, CYAN_DIM, TEXT_1, TEXT_2, TEXT_3,
};
use crate::ui::{Icon, IconName, mono, section_title};
use crate::views::grid::{Col, cell};
use crate::views::intel_card::{Close, intel_panel};
use crate::views::motion::{self, Arrivals};
use crate::views::notes::annotation_notes_button;
use crate::views::pilot_details::{DETAILS_HEIGHT, pilot_details};
use crate::views::stats::danger_cell;
use crate::views::tags::{TABLE, role_glyph, tag_strip};

const ROW_HEIGHT: f32 = 40.;

const COLS: [Col; 7] = [
    Col::Fixed(64.),
    Col::Grow(180., 1.6),
    Col::Grow(150., 1.2),
    Col::Grow(140., 1.2),
    Col::Grow(140., 1.2),
    Col::Fixed(168.),
    Col::Fixed(140.),
];

pub struct LocalPanel {
    sort: SortState,
    expanded: Option<u64>,
    expanded_at: Option<Instant>,
    rows: Vec<PilotIntel>,
    /// Where each row starts in the list, for sliding rows that move.
    row_tops: Vec<f32>,
    arrived: Arrivals,
    scored: Arrivals,
    row_sizes: Rc<Vec<Size<Pixels>>>,
    scroll: VirtualListScrollHandle,
    /// The pilot whose intel panel is open, and where it was opened.
    menu: Option<(u64, Point<Pixels>)>,
    menu_focus: FocusHandle,
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
            expanded_at: None,
            rows: Vec::new(),
            row_tops: Vec::new(),
            arrived: Arrivals::default(),
            scored: Arrivals::default(),
            row_sizes: Rc::new(Vec::new()),
            scroll: VirtualListScrollHandle::new(),
            menu: None,
            menu_focus: cx.focus_handle(),
            _subscriptions: subscriptions,
        };
        panel.refresh(cx);
        panel
    }

    /// Re-filters and re-sorts the rows and recomputes their heights.
    fn refresh(&mut self, cx: &mut Context<Self>) {
        let stores = Stores::get(cx);
        let filters = stores.filters.read(cx);
        let pilots = stores.scan.read(cx).pilots();
        self.arrived.sync(pilots.iter().map(PilotIntel::row_key));
        self.scored.sync(
            pilots
                .iter()
                .filter(|p| p.zkill.is_some())
                .map(PilotIntel::row_key),
        );
        let mut rows: Vec<PilotIntel> = pilots
            .iter()
            .filter(|p| filters.matches(p, cx))
            .cloned()
            .collect();
        sort_pilots(&mut rows, self.sort.key, self.sort.direction);
        let heights: Vec<f32> = rows.iter().map(|p| self.row_height(p)).collect();
        self.row_tops = heights
            .iter()
            .scan(0., |top, height| {
                let this = *top;
                *top += height;
                Some(this)
            })
            .collect();
        self.row_sizes = Rc::new(heights.iter().map(|h| size(px(1.), px(*h))).collect());
        self.rows = rows;
        cx.notify();
    }

    fn row_height(&self, pilot: &PilotIntel) -> f32 {
        let mut height = ROW_HEIGHT;
        if self.expanded == Some(pilot.row_key()) && pilot.zkill.is_some() {
            height += DETAILS_HEIGHT;
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

    fn toggle_expand(&mut self, key: u64, cx: &mut Context<Self>) {
        self.expanded = if self.expanded == Some(key) {
            None
        } else {
            Some(key)
        };
        self.expanded_at = Some(Instant::now());
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
            .child(header_cell(COLS[6]).child(self.sort_header(
                SortKey::Danger,
                "DANGER",
                true,
                cx,
            )))
    }

    fn render_row(
        &self,
        pilot: &PilotIntel,
        top: f32,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let id = pilot.character.id;
        let key = pilot.row_key();
        let expanded = self.expanded == Some(key) && pilot.zkill.is_some();
        let intel = Stores::get(cx).intel.read(cx);
        let resolved = intel.resolve(pilot);
        let tags = pilot_tags(pilot, &resolved);
        let notes = annotation_notes_button(pilot, &resolved);
        let threat = theme::threat_color(&pilot.threat_level);
        let row_cell = |col: Col| cell(col).px_2().py_1p5();
        let dash = || div().text_color(theme::color(TEXT_3)).child("—");
        let z = pilot.zkill.as_ref();

        let main_row = if pilot.is_unresolved() {
            unresolved_row(pilot)
        } else {
            div()
                .id(("pilot-row", key))
                .flex()
                .h(px(ROW_HEIGHT - 1.))
                .bg(theme::color(BG_1))
                .hover(|s| s.bg(theme::color(BG_HOVER)))
                .border_l(px(3.))
                .border_color(threat)
                .cursor_pointer()
                .on_click(cx.listener(move |this, _, _, cx| this.toggle_expand(key, cx)))
                .on_mouse_down(
                    MouseButton::Right,
                    cx.listener(move |this, event: &MouseDownEvent, window, cx| {
                        // Unresolved names have no character to annotate.
                        if id == 0 {
                            return;
                        }
                        this.menu = Some((key, event.position));
                        window.focus(&this.menu_focus, cx);
                        cx.notify();
                    }),
                )
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
                    row_cell(COLS[2])
                        .gap_1()
                        .child(tag_strip("row-tag", key, &tags, TABLE))
                        .children(notes),
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
                                    .id(gpui_kit::ElementId::NamedInteger(
                                        format!("row-ship-{}", ship.ship_type_id).into(),
                                        key,
                                    ))
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
                .child(row_cell(COLS[6]).child(danger_cell(
                    ("row-danger", key).into(),
                    pilot,
                    80.,
                    px(14.),
                    self.scored.is_fresh(key),
                )))
        };

        let height = self.row_height(pilot);
        let row = div()
            .w_full()
            .h(px(height))
            .flex()
            .flex_col()
            .border_b_1()
            .border_color(theme::color(BORDER))
            .child(main_row)
            .when_some(z.filter(|_| expanded), |el, z| {
                el.child(motion::enter(
                    ("details-in", key),
                    div().child(pilot_details(pilot, z, cx)),
                    self.expanded_at.is_some_and(motion::is_recent),
                ))
            });
        let offset = motion::slide(("row-y", key), top, window, cx);
        div()
            .h(px(height))
            .relative()
            .top(offset)
            .child(motion::enter(
                ("row-in", key),
                row,
                self.arrived.is_fresh(key),
            ))
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
                        // Borrow the rows out instead of cloning them every frame.
                        let rows = std::mem::take(&mut this.rows);
                        let elements = range
                            .map(|ix| this.render_row(&rows[ix], this.row_tops[ix], window, cx))
                            .collect();
                        this.rows = rows;
                        elements
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
                                            .children(role_glyph(&t.tag, &color, px(14.)))
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

impl LocalPanel {
    fn close_menu(&mut self, cx: &mut Context<Self>) {
        if self.menu.take().is_some() {
            cx.notify();
        }
    }

    fn render_menu(&self, cx: &mut Context<Self>) -> Option<impl IntoElement> {
        let (id, position) = self.menu?;
        let pilot = self.rows.iter().find(|p| p.row_key() == id)?;
        let this = cx.entity().downgrade();
        let close: Close = Rc::new(move |_, cx| {
            let _ = this.update(cx, |this, cx| this.close_menu(cx));
        });
        Some(
            deferred(
                anchored()
                    .position(position)
                    .snap_to_window_with_margin(px(8.))
                    .child(
                        div()
                            .track_focus(&self.menu_focus)
                            .on_mouse_down_out(cx.listener(|this, _, _, cx| this.close_menu(cx)))
                            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                                if event.keystroke.key == "escape" {
                                    this.close_menu(cx);
                                }
                            }))
                            .child(motion::enter(
                                ("intel-in", id),
                                div().child(intel_panel(pilot, close, cx)),
                                true,
                            )),
                    ),
            )
            .with_priority(1),
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
            .children(self.render_menu(cx))
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
    order.sort_by_key(|g| std::cmp::Reverse(g.count));
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

/// A muted row for a name the lookup could not resolve: no stats to show, so
/// one line says why instead of a column of dashes.
fn unresolved_row(pilot: &PilotIntel) -> Stateful<Div> {
    let id = pilot.character.id;
    let (icon, reason): (IconName, SharedString) = if id == 0 {
        (IconName::UserX, "Unknown character".into())
    } else {
        (IconName::CloudOff, "Lookup failed".into())
    };
    let detail: SharedString = pilot.error.clone().unwrap_or_default().into();
    div()
        .id(("pilot-row", pilot.row_key()))
        .flex()
        .items_center()
        .h(px(ROW_HEIGHT - 1.))
        .bg(theme::color(BG_0))
        .border_l(px(3.))
        .border_color(theme::color(BORDER))
        .text_color(theme::color(TEXT_3))
        .tooltip(move |window, cx| {
            gpui_kit::component::tooltip::Tooltip::new(detail.clone()).build(window, cx)
        })
        .child(
            cell(COLS[0]).px_2().child(
                div()
                    .px_1p5()
                    .rounded_sm()
                    .border_1()
                    .border_color(theme::color(BORDER))
                    .text_size(px(10.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("N/A"),
            ),
        )
        .child(
            cell(COLS[1]).px_2().child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .min_w_0()
                    .child(div().opacity(0.6).child(portrait(id)))
                    .child(
                        div()
                            .truncate()
                            .text_sm()
                            .text_color(theme::color(TEXT_2))
                            .child(pilot.character.name.clone()),
                    ),
            ),
        )
        .child(
            cell(COLS[2])
                .px_2()
                .gap_2()
                .text_xs()
                .child(Icon::new(icon).size_3p5().flex_none())
                .child(div().truncate().child(reason)),
        )
        .children(COLS[3..].iter().map(|col| cell(*col)))
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
