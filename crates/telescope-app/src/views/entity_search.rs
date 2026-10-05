use std::time::Duration;

use gpui_kit::component::Sizable as _;
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AppContext as _, Context, Entity, EventEmitter, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, SharedString, StatefulInteractiveElement as _, Styled as _,
    Subscription, Task, Window, div, img, px,
};
use telescope_core::models::SearchResult;
use telescope_core::view::annotations::EntityType;
use telescope_core::view::network::portrait_url;

use crate::state::Stores;
use crate::theme::{self, BG_2, BG_HOVER, BORDER, CYAN, TEXT_1, TEXT_3};

const DEBOUNCE: Duration = Duration::from_millis(250);

pub enum EntitySearchEvent {
    Selected(SearchResult),
}

/// Character, corporation and alliance search against the Telescope API.
pub struct EntitySearch {
    category: Option<EntityType>,
    query: Entity<InputState>,
    results: Vec<SearchResult>,
    selected: Option<SearchResult>,
    search: Option<Task<()>>,
    _subscription: Subscription,
}

impl EventEmitter<EntitySearchEvent> for EntitySearch {}

impl EntitySearch {
    pub fn new(category: Option<EntityType>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let query = cx.new(|cx| InputState::new(window, cx).placeholder("Search entity..."));
        let subscription = cx.subscribe(&query, |this, query, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                let text = query.read(cx).value().to_string();
                this.schedule(text, cx);
            }
        });
        Self {
            category,
            query,
            results: Vec::new(),
            selected: None,
            search: None,
            _subscription: subscription,
        }
    }

    /// Replacing the task cancels the previous debounce and any request still
    /// in flight, so stale results never overwrite newer ones.
    fn schedule(&mut self, text: String, cx: &mut Context<Self>) {
        if text.trim().chars().count() < 2 {
            self.search = None;
            self.results.clear();
            cx.notify();
            return;
        }
        let category = self.category;
        self.search = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(DEBOUNCE).await;
            let Ok(request) = this.update(cx, |_, cx| {
                Stores::get(cx)
                    .intel
                    .read(cx)
                    .search_entities(text, category)
            }) else {
                return;
            };
            let results = request.await.unwrap_or_default();
            let _ = this.update(cx, |this, cx| {
                this.results = results;
                cx.notify();
            });
        }));
    }

    fn select(&mut self, result: SearchResult, window: &mut Window, cx: &mut Context<Self>) {
        self.results.clear();
        self.search = None;
        self.query
            .update(cx, |state, cx| state.set_value("", window, cx));
        cx.emit(EntitySearchEvent::Selected(result.clone()));
        self.selected = Some(result);
        cx.notify();
    }
}

fn affiliation_line(result: &SearchResult) -> Option<String> {
    let corp = result
        .corporation
        .as_ref()
        .map(|c| format!("[{}] {}", c.ticker, c.name));
    let alliance = result
        .alliance
        .as_ref()
        .map(|a| format!("<{}> {}", a.ticker, a.name));
    match (corp, alliance) {
        (Some(c), Some(a)) => Some(format!("{c} · {a}")),
        (c, a) => c.or(a),
    }
}

impl Render for EntitySearch {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_1()
            .when_some(self.selected.clone(), |el, selected| {
                el.child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .px_2()
                        .py_1p5()
                        .rounded_sm()
                        .bg(theme::color(BG_2))
                        .border_1()
                        .border_color(theme::color(BORDER))
                        .text_xs()
                        .when_some(
                            portrait_url(&selected.category, selected.id, 64),
                            |el, url| el.child(img(url).size(px(20.)).rounded_sm()),
                        )
                        .child(div().flex_1().truncate().child(selected.name.clone()))
                        .child(
                            div()
                                .text_size(px(9.))
                                .text_color(theme::color(TEXT_3))
                                .child(selected.category.to_uppercase()),
                        ),
                )
            })
            .child(Input::new(&self.query).small())
            .when(!self.results.is_empty(), |el| {
                el.child(
                    div()
                        .id("entity-results")
                        .max_h(px(192.))
                        .overflow_y_scroll()
                        .rounded_sm()
                        .border_1()
                        .border_color(theme::color(BORDER))
                        .bg(theme::color(BG_2))
                        .children(self.results.iter().map(|result| {
                            let key: SharedString =
                                format!("entity-{}-{}", result.category, result.id).into();
                            let picked = result.clone();
                            div()
                                .id(key)
                                .flex()
                                .items_center()
                                .gap_2()
                                .px_3()
                                .py_2()
                                .cursor_pointer()
                                .hover(|s| s.bg(theme::color(BG_HOVER)))
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.select(picked.clone(), window, cx)
                                }))
                                .when_some(
                                    portrait_url(&result.category, result.id, 64),
                                    |el, url| {
                                        el.child(img(url).size(px(24.)).rounded_sm().flex_none())
                                    },
                                )
                                .child(
                                    div()
                                        .min_w_0()
                                        .child(
                                            div()
                                                .flex()
                                                .gap_1()
                                                .text_xs()
                                                .text_color(theme::color(TEXT_1))
                                                .child(div().truncate().child(result.name.clone()))
                                                .child(
                                                    div()
                                                        .text_size(px(9.))
                                                        .text_color(theme::color(CYAN))
                                                        .child(result.category.to_uppercase()),
                                                )
                                                .when_some(result.ticker.clone(), |el, t| {
                                                    el.child(
                                                        div()
                                                            .text_color(theme::color(TEXT_3))
                                                            .child(format!("[{t}]")),
                                                    )
                                                }),
                                        )
                                        .when_some(affiliation_line(result), |el, line| {
                                            el.child(
                                                div()
                                                    .truncate()
                                                    .text_size(px(9.))
                                                    .text_color(theme::color(TEXT_3))
                                                    .child(line),
                                            )
                                        }),
                                )
                        })),
                )
            })
    }
}
