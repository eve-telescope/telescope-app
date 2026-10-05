//! The compact always-on-top overlay. It renders the same scan and filter
//! state as the main window, so there is nothing to sync between them.

use std::rc::Rc;

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{Selectable as _, Sizable as _, VirtualListScrollHandle, v_virtual_list};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AnyWindowHandle, App, AppContext as _, Bounds, Context, FontWeight, Global,
    InteractiveElement as _, IntoElement, MouseButton, ParentElement as _, Pixels, Render,
    SharedString, Size, StatefulInteractiveElement as _, Styled as _, Subscription, Window,
    WindowBackgroundAppearance, WindowBounds, WindowKind, WindowOptions, div, img, point, px, size,
};
use telescope_core::models::PilotIntel;
use telescope_core::view::format::ship_icon_url;
use telescope_core::view::pilot_counts::{tag_counts, threat_counts};
use telescope_core::view::pilot_sort::{SortDirection, SortKey, SortState, sort_pilots};
use telescope_core::view::pilot_tags::{
    DEFAULT_TAG_COLOR, DEFAULT_TAG_TEXT_COLOR, pilot_tags_with_index,
};

use crate::state::Stores;
use crate::theme::{self, BG_0, BG_1, BG_2, BG_3, BG_HOVER, BORDER, CYAN, TEXT_1, TEXT_2, TEXT_3};
use crate::ui::{Icon, IconName, dot};
use crate::views::grid::{Col, cell};
use crate::views::local_panel::portrait;
use crate::views::motion::{self, Arrivals};
use crate::views::pilot_details::zkill_character_url;
use crate::views::stats::danger_cell;
use crate::views::tags::{OVERLAY, role_glyph, tag_strip};

const ROW_HEIGHT: f32 = 33.;

const COLS: [Col; 7] = [
    Col::Fixed(32.),
    Col::Grow(80., 1.),
    Col::Grow(70., 0.6),
    Col::Fixed(45.),
    Col::Fixed(45.),
    Col::Fixed(60.),
    Col::Fixed(76.),
];

struct OverlayWindow(AnyWindowHandle);

impl Global for OverlayWindow {}

pub struct OverlayView {
    sort: SortState,
    rows: Vec<PilotIntel>,
    arrived: Arrivals,
    scored: Arrivals,
    row_sizes: Rc<Vec<Size<Pixels>>>,
    scroll: VirtualListScrollHandle,
    _subscriptions: Vec<Subscription>,
}

impl OverlayView {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let stores = Stores::get(cx);
        let subscriptions = vec![
            cx.observe(&stores.scan, |this, _, cx| this.refresh(cx)),
            cx.observe(&stores.filters, |this, _, cx| this.refresh(cx)),
            cx.observe(&stores.intel, |this, _, cx| this.refresh(cx)),
            cx.observe(&stores.settings, |_, _, cx| cx.notify()),
            cx.observe_window_bounds(window, |_, window, cx| save_bounds(window, cx)),
        ];
        let mut view = Self {
            sort: SortState::default(),
            rows: Vec::new(),
            arrived: Arrivals::default(),
            scored: Arrivals::default(),
            row_sizes: Rc::new(Vec::new()),
            scroll: VirtualListScrollHandle::new(),
            _subscriptions: subscriptions,
        };
        view.refresh(cx);
        view
    }

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
        self.row_sizes = Rc::new(rows.iter().map(|_| size(px(1.), px(ROW_HEIGHT))).collect());
        self.rows = rows;
        cx.notify();
    }

    fn header(
        &self,
        key: SortKey,
        label: &'static str,
        right: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let active = self.sort.key == key;
        let arrow = match self.sort.direction {
            SortDirection::Asc => "↑",
            SortDirection::Desc => "↓",
        };
        div()
            .id(SharedString::from(format!("overlay-sort-{}", key.as_str())))
            .flex()
            .gap_0p5()
            .when(right, |el| el.ml_auto())
            .cursor_pointer()
            .when(active, |el| el.text_color(theme::color(CYAN)))
            .hover(|s| s.text_color(theme::color(TEXT_1)))
            .child(label)
            .when(active, |el| el.child(div().text_size(px(8.)).child(arrow)))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.sort.toggle(key);
                this.refresh(cx);
            }))
    }

    fn render_header_bar(&self, locked: bool, count: usize) -> impl IntoElement {
        div()
            .id("overlay-header")
            .flex()
            .flex_none()
            .items_center()
            .justify_between()
            .px_3()
            .py_1p5()
            .bg(theme::color(BG_1))
            .border_b_1()
            .border_color(theme::tint(CYAN, 0x4d))
            .when(!locked, |el| {
                el.on_mouse_down(MouseButton::Left, |_, window, _| window.start_window_move())
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(dot(theme::color(CYAN), 8.))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme::color(CYAN))
                            .child(count.to_string()),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(
                        Button::new("overlay-lock")
                            .ghost()
                            .xsmall()
                            .icon(Icon::new(if locked {
                                IconName::Lock
                            } else {
                                IconName::LockOpen
                            }))
                            .selected(locked)
                            .when(locked, |b| b.text_color(theme::color(CYAN)))
                            .tooltip(if locked {
                                "Unlock position"
                            } else {
                                "Lock position"
                            })
                            .on_click(|_, _, cx| set_locked(!is_locked(cx), cx)),
                    )
                    .child(
                        Button::new("overlay-close")
                            .ghost()
                            .xsmall()
                            .icon(IconName::X)
                            .tooltip("Close overlay")
                            .on_click(|_, window, _| window.remove_window()),
                    ),
            )
    }

    fn render_filters(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let stores = Stores::get(cx);
        let pilots = stores.scan.read(cx).pilots();
        let index = stores.intel.read(cx).annotation_index();
        let threat = threat_counts(pilots);
        let tags = tag_counts(pilots, |p| pilot_tags_with_index(p, index));
        let filters = stores.filters.read(cx).get();
        let active_threat = filters.threat.clone();
        let selected_tags = filters.selected_tags.clone();

        let threat_button = |level: &'static str, count: usize| {
            let color = theme::threat_color(level);
            let active = active_threat.as_deref() == Some(level);
            div()
                .id(SharedString::from(format!("overlay-threat-{level}")))
                .flex()
                .items_center()
                .gap_1()
                .px_1p5()
                .py_0p5()
                .rounded_sm()
                .cursor_pointer()
                .map(|el| {
                    if active {
                        el.bg(color.opacity(0.3))
                    } else {
                        el.hover(|s| s.bg(theme::color(BG_HOVER)))
                    }
                })
                .on_click(move |_, _, cx| {
                    Stores::get(cx)
                        .filters
                        .update(cx, |f, cx| f.update(cx, |f| f.toggle_threat(level)))
                })
                .child(dot(color, 8.))
                .child(
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_color(color)
                        .child(count.to_string()),
                )
        };

        div()
            .flex()
            .flex_none()
            .items_center()
            .justify_between()
            .gap_2()
            .px_3()
            .py_1()
            .bg(theme::color(BG_1))
            .border_b_1()
            .border_color(theme::tint(BORDER, 0x80))
            .text_size(px(10.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .flex_none()
                    .when(threat.extreme > 0, |el| {
                        el.child(threat_button("extreme", threat.extreme))
                    })
                    .when(threat.high > 0, |el| {
                        el.child(threat_button("high", threat.high))
                    })
                    .when(threat.moderate > 0, |el| {
                        el.child(threat_button("moderate", threat.moderate))
                    })
                    .when(threat.low > 0, |el| {
                        el.child(threat_button("low", threat.low))
                    })
                    .when(threat.minimal > 0, |el| {
                        el.child(threat_button("minimal", threat.minimal))
                    }),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .justify_end()
                    .items_center()
                    .gap_1()
                    .children(tags.into_iter().map(|t| {
                        let selected = selected_tags.contains(&t.tag);
                        let color = t.color.clone().unwrap_or_else(|| DEFAULT_TAG_COLOR.into());
                        let text = t
                            .color
                            .clone()
                            .unwrap_or_else(|| DEFAULT_TAG_TEXT_COLOR.into());
                        let tag = t.tag.clone();
                        div()
                            .id(SharedString::from(format!("overlay-tag-{}", t.tag)))
                            .flex()
                            .items_center()
                            .gap_1()
                            .px_1p5()
                            .py_0p5()
                            .rounded_sm()
                            .font_weight(FontWeight::BOLD)
                            .cursor_pointer()
                            .bg(theme::hex_tint(
                                &color,
                                if selected { 0x66 } else { 0x33 },
                                TEXT_3,
                            ))
                            .text_color(theme::hex_or(&text, TEXT_2))
                            .on_click(move |_, _, cx| {
                                Stores::get(cx)
                                    .filters
                                    .update(cx, |f, cx| f.update(cx, |f| f.toggle_tag(&tag)))
                            })
                            .child(t.count.to_string())
                            .map(|el| match role_glyph(&t.tag, &color, px(12.)) {
                                Some(icon) => el.child(icon),
                                None => el.child(t.tag.clone()),
                            })
                    })),
            )
    }

    fn render_columns(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let c = |col: Col| cell(col).px_0p5();
        div()
            .flex()
            .flex_none()
            .px_3()
            .py_1()
            .bg(theme::color(BG_2))
            .border_b_1()
            .border_color(theme::tint(BORDER, 0x80))
            .text_size(px(9.))
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(theme::color(TEXT_3))
            .child(
                c(COLS[0])
                    .justify_center()
                    .child(self.header(SortKey::Threat, "⬤", false, cx)),
            )
            .child(c(COLS[1]).child(self.header(SortKey::Pilot, "PILOT", false, cx)))
            .child(c(COLS[2]).child(self.header(SortKey::Tags, "TAGS", false, cx)))
            .child(c(COLS[3]).child(self.header(SortKey::Corp, "CORP", false, cx)))
            .child(c(COLS[4]).child(self.header(SortKey::Alliance, "ALLY", false, cx)))
            .child(c(COLS[5]).child("SHIPS"))
            .child(c(COLS[6]).child(self.header(SortKey::Danger, "DANGER", true, cx)))
    }

    fn render_row(
        &self,
        pilot: &PilotIntel,
        top: f32,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> gpui_kit::AnyElement {
        let id = pilot.character.id;
        let key = pilot.row_key();
        let index = Stores::get(cx).intel.read(cx).annotation_index();
        let tags = pilot_tags_with_index(pilot, index);
        let z = pilot.zkill.as_ref();
        let c = |col: Col| cell(col).px_0p5();
        let dash = || div().text_color(theme::color(TEXT_3)).child("—");
        let ticker = |ticker: &Option<String>, name: &Option<String>, label: &'static str| {
            let tooltip: SharedString = name.clone().unwrap_or_default().into();
            div()
                .id(SharedString::from(format!("{label}-{key}")))
                .truncate()
                .text_size(px(10.))
                .text_color(theme::color(TEXT_3))
                .when(!tooltip.is_empty(), |el| {
                    el.tooltip(move |window, cx| {
                        gpui_kit::component::tooltip::Tooltip::new(tooltip.clone())
                            .build(window, cx)
                    })
                })
                .child(ticker.clone().unwrap_or_else(|| "—".into()))
        };

        let row = div()
            .id(("overlay-row", key))
            .flex()
            .w_full()
            .items_center()
            .h(px(ROW_HEIGHT))
            .px_3()
            .border_b_1()
            .border_color(theme::tint(BORDER, 0x33))
            .when(pilot.is_unresolved(), |el| el.opacity(0.4))
            .cursor_pointer()
            .hover(|s| s.bg(theme::tint(BG_HOVER, 0x80)))
            .on_click(move |_, _, cx| {
                if id != 0 {
                    cx.open_url(&zkill_character_url(id))
                }
            })
            .child(
                c(COLS[0])
                    .justify_center()
                    .child(dot(theme::threat_color(&pilot.threat_level), 8.)),
            )
            .child(
                c(COLS[1])
                    .gap_1p5()
                    .child(div().size(px(24.)).flex_none().child(portrait(id)))
                    .child(
                        div()
                            .truncate()
                            .text_size(px(11.))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme::color(TEXT_1))
                            .child(pilot.character.name.clone()),
                    ),
            )
            .child(c(COLS[2]).child(tag_strip("overlay-tag", key, &tags, OVERLAY)))
            .child(c(COLS[3]).child(ticker(
                &pilot.character.corporation_ticker,
                &pilot.character.corporation_name,
                "overlay-corp",
            )))
            .child(c(COLS[4]).child(ticker(
                &pilot.character.alliance_ticker,
                &pilot.character.alliance_name,
                "overlay-ally",
            )))
            .child(
                c(COLS[5])
                    .gap_0p5()
                    .child(match z.filter(|z| !z.top_ships.is_empty()) {
                        Some(z) => div()
                            .flex()
                            .items_center()
                            .gap_0p5()
                            .children(z.top_ships.iter().take(3).map(|ship| {
                                let url = format!(
                                    "https://zkillboard.com/character/{id}/ship/{}/",
                                    ship.ship_type_id
                                );
                                let tooltip: SharedString = ship.ship_name.clone().into();
                                div()
                                    .id(gpui_kit::ElementId::NamedInteger(
                                        format!("overlay-ship-{}", ship.ship_type_id).into(),
                                        key,
                                    ))
                                    .size(px(18.))
                                    .rounded_sm()
                                    .overflow_hidden()
                                    .bg(theme::color(BG_3))
                                    .hover(|s| s.border_1().border_color(theme::color(CYAN)))
                                    .tooltip(move |window, cx| {
                                        gpui_kit::component::tooltip::Tooltip::new(tooltip.clone())
                                            .build(window, cx)
                                    })
                                    .on_click(move |_, _, cx| {
                                        cx.stop_propagation();
                                        cx.open_url(&url)
                                    })
                                    .child(img(ship_icon_url(ship.ship_type_id, 32)).size_full())
                            }))
                            .when(z.top_ships.len() > 3, |el| {
                                el.child(
                                    div()
                                        .text_size(px(9.))
                                        .text_color(theme::color(TEXT_3))
                                        .child(format!("+{}", z.top_ships.len() - 3)),
                                )
                            })
                            .into_any_element(),
                        None => dash().into_any_element(),
                    }),
            )
            .child(c(COLS[6]).child(danger_cell(
                ("overlay-danger", key).into(),
                pilot,
                36.,
                px(11.),
                self.scored.is_fresh(key),
            )));
        let offset = motion::slide(("overlay-row-y", key), top, window, cx);
        div()
            .h(px(ROW_HEIGHT))
            .relative()
            .top(offset)
            .child(motion::enter(
                ("overlay-row-in", key),
                row,
                self.arrived.is_fresh(key),
            ))
            .into_any_element()
    }
}

impl Render for OverlayView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let count = Stores::get(cx).scan.read(cx).pilots().len();
        let locked = is_locked(cx);
        div()
            .size_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(theme::color(BG_0))
            .text_color(theme::color(TEXT_1))
            .child(self.render_header_bar(locked, count))
            .when(count > 0, |el| {
                el.child(self.render_filters(cx))
                    .child(self.render_columns(cx))
                    .child(
                        v_virtual_list(
                            cx.entity(),
                            "overlay-rows",
                            self.row_sizes.clone(),
                            |this, range, window, cx| {
                                let rows = std::mem::take(&mut this.rows);
                                let elements = range
                                    .map(|ix| {
                                        let top = ix as f32 * ROW_HEIGHT;
                                        this.render_row(&rows[ix], top, window, cx)
                                    })
                                    .collect();
                                this.rows = rows;
                                elements
                            },
                        )
                        .track_scroll(&self.scroll)
                        .flex_1(),
                    )
            })
    }
}

fn is_locked(cx: &App) -> bool {
    Stores::get(cx).settings.read(cx).get().overlay_locked
}

fn save_bounds(window: &mut Window, cx: &mut App) {
    if let WindowBounds::Windowed(bounds) = window.window_bounds() {
        Stores::get(cx).settings.update(cx, |settings, cx| {
            settings.update(cx, |s| {
                s.overlay_window = Some(telescope_core::settings::WindowBounds {
                    x: bounds.origin.x.into(),
                    y: bounds.origin.y.into(),
                    width: bounds.size.width.into(),
                    height: bounds.size.height.into(),
                })
            })
        });
    }
}

/// GPUI can't change whether an open window is movable or resizable, so
/// locking reopens the overlay in place with the new options. This runs from
/// the overlay's own click handler, where the window can't be removed, hence
/// the deferral.
fn set_locked(locked: bool, cx: &mut App) {
    Stores::get(cx).settings.update(cx, |settings, cx| {
        settings.update(cx, |s| s.overlay_locked = locked)
    });
    if is_open(cx) {
        cx.defer(|cx| {
            close(cx);
            cx.defer(create);
        });
    }
}

pub fn is_open(cx: &App) -> bool {
    cx.try_global::<OverlayWindow>()
        .is_some_and(|w| cx.windows().contains(&w.0))
}

pub fn open(cx: &mut App) {
    if let Some(OverlayWindow(handle)) = cx.try_global::<OverlayWindow>()
        && cx.windows().contains(handle)
    {
        let handle = *handle;
        let _ = handle.update(cx, |_, window, _| window.activate_window());
        return;
    }
    create(cx);
}

fn create(cx: &mut App) {
    let settings = Stores::get(cx).settings.read(cx).get().clone();
    let bounds = match settings.overlay_window {
        Some(b) => Bounds::new(point(px(b.x), px(b.y)), size(px(b.width), px(b.height))),
        None => Bounds::centered(None, size(px(580.), px(450.)), cx),
    };
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        window_min_size: Some(size(px(480.), px(200.))),
        titlebar: None,
        kind: WindowKind::PopUp,
        is_resizable: !settings.overlay_locked,
        is_movable: !settings.overlay_locked,
        is_minimizable: false,
        window_background: WindowBackgroundAppearance::Opaque,
        ..Default::default()
    };
    match gpui_kit::open_window(options, cx, |window, cx| {
        window.set_window_title("Telescope Overlay");
        cx.new(|cx| OverlayView::new(window, cx))
    }) {
        Ok((handle, _)) => cx.set_global(OverlayWindow(handle)),
        Err(e) => log::error!("Failed to open overlay: {}", e),
    }
    cx.refresh_windows();
}

pub fn close(cx: &mut App) {
    if let Some(OverlayWindow(handle)) = cx.try_global::<OverlayWindow>() {
        let handle = *handle;
        let _ = handle.update(cx, |_, window, _| window.remove_window());
    }
    cx.refresh_windows();
}

pub fn toggle(cx: &mut App) {
    if is_open(cx) { close(cx) } else { open(cx) }
}
