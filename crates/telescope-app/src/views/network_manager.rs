use chrono::Utc;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::radio::RadioGroup;
use gpui_kit::component::{Disableable as _, Selectable as _, Sizable as _, WindowExt as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    App, AppContext as _, Context, Entity, FontWeight, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, SharedString, StatefulInteractiveElement as _, Styled as _,
    Subscription, Task, Window, div, img, px,
};
use telescope_core::intel_service::AccessInput;
use telescope_core::models::{NetworkAccess, NetworkDetail, NetworkScan, PaginatedScans};
use telescope_core::view::annotations::{Annotation, DEFAULT_ANNOTATION_COLOR};
use telescope_core::view::format::relative_time;
use telescope_core::view::network::{
    PermissionLevel, access_permission_label, can_remove_access, describe_permission, portrait_url,
};

use crate::state::Stores;
use crate::state::intel::IntelEvent;
use crate::theme::{
    self, BG_0, BG_1, BG_2, BG_3, BG_HOVER, BORDER, CYAN, GREEN, RED, TEXT_1, TEXT_2, TEXT_3,
};
use crate::ui::{Icon, IconName};
use crate::views::annotation_form::{self, Target};
use crate::views::entity_search::{EntitySearch, EntitySearchEvent};

#[derive(Clone, Copy, PartialEq)]
enum DetailTab {
    Annotations,
    Scans,
    Access,
}

pub struct NetworkManager {
    detail: bool,
    /// Set once the user goes back to the list, so the active network
    /// doesn't reopen on its own.
    left_detail: bool,
    tab: DetailTab,
    error: Option<String>,
    new_network: Entity<InputState>,
    scans: Option<PaginatedScans>,
    scans_page: i64,
    scans_loading: bool,
    scans_task: Option<Task<()>>,
    scans_network: Option<i64>,
    _subscriptions: Vec<Subscription>,
}

impl NetworkManager {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let stores = Stores::get(cx);
        let new_network = cx.new(|cx| InputState::new(window, cx).placeholder("New network name"));
        let subscriptions = vec![
            cx.subscribe_in(&new_network, window, |this, _, event, window, cx| {
                if matches!(event, InputEvent::PressEnter { .. }) {
                    this.create_network(window, cx);
                }
            }),
            cx.observe(&stores.intel, |this, intel, cx| {
                // Opening the tab shows the active network, like the Vue app.
                let intel = intel.read(cx);
                if !intel.is_authenticated() {
                    this.detail = false;
                } else if !this.detail
                    && let (Some(active), Some(selected)) =
                        (intel.active_network_id(), intel.selected_network())
                    && selected.id == active
                    && !this.left_detail
                {
                    this.detail = true;
                }
                if this.detail && this.tab == DetailTab::Scans {
                    this.sync_scans(cx);
                }
                cx.notify();
            }),
            cx.subscribe(&stores.intel, |this, _, event, cx| match event {
                IntelEvent::Error(e) => {
                    this.error = Some(e.clone());
                    cx.notify();
                }
                IntelEvent::ScanShared(scan) => {
                    if this.scans_page == 1
                        && let Some(page) = &mut this.scans
                    {
                        page.data.insert(0, scan.clone());
                        cx.notify();
                    }
                }
            }),
        ];
        Self {
            detail: false,
            left_detail: false,
            tab: DetailTab::Annotations,
            error: None,
            new_network,
            scans: None,
            scans_page: 1,
            scans_loading: false,
            scans_task: None,
            scans_network: None,
            _subscriptions: subscriptions,
        }
    }

    fn create_network(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let name = self.new_network.read(cx).value().trim().to_string();
        if name.is_empty() {
            return;
        }
        self.new_network
            .update(cx, |state, cx| state.set_value("", window, cx));
        Stores::get(cx)
            .intel
            .update(cx, |intel, cx| intel.create_network(name, cx));
    }

    fn open_detail(&mut self, network_id: i64, cx: &mut Context<Self>) {
        self.detail = true;
        self.tab = DetailTab::Annotations;
        self.scans = None;
        self.scans_network = None;
        Stores::get(cx)
            .intel
            .update(cx, |intel, cx| intel.select_network(network_id, cx));
        cx.notify();
    }

    fn back(&mut self, cx: &mut Context<Self>) {
        self.detail = false;
        self.left_detail = true;
        Stores::get(cx)
            .intel
            .update(cx, |intel, cx| intel.clear_selected_network(cx));
        cx.notify();
    }

    fn set_tab(&mut self, tab: DetailTab, cx: &mut Context<Self>) {
        self.tab = tab;
        if tab == DetailTab::Scans {
            self.sync_scans(cx);
        }
        cx.notify();
    }

    fn sync_scans(&mut self, cx: &mut Context<Self>) {
        let selected = Stores::get(cx)
            .intel
            .read(cx)
            .selected_network()
            .map(|n| n.id);
        if selected.is_some() && selected != self.scans_network {
            self.scans_network = selected;
            self.scans_page = 1;
            self.load_scans(cx);
        }
    }

    fn load_scans(&mut self, cx: &mut Context<Self>) {
        let Some(network_id) = self.scans_network else {
            return;
        };
        self.scans_loading = true;
        let fetch = Stores::get(cx)
            .intel
            .read(cx)
            .fetch_scans(network_id, self.scans_page);
        self.scans_task = Some(cx.spawn(async move |this, cx| {
            let result = fetch.await;
            let _ = this.update(cx, |this, cx| {
                match result {
                    Ok(page) => this.scans = Some(page),
                    Err(e) => this.error = Some(e),
                }
                this.scans_loading = false;
                cx.notify();
            });
        }));
        cx.notify();
    }

    fn render_unauthenticated(&self) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .px_6()
            .child(
                div()
                    .size(px(40.))
                    .mb_3()
                    .rounded_full()
                    .bg(theme::color(BG_2))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        Icon::new(IconName::Network)
                            .size_5()
                            .text_color(theme::color(TEXT_3)),
                    ),
            )
            .child(
                div()
                    .mb_1()
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Intel Networks"),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(theme::color(TEXT_3))
                    .child("Sign in via the Settings tab to access your intel networks."),
            )
    }

    fn render_list(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let intel = Stores::get(cx).intel.read(cx);
        let active = intel.active_network_id();
        let networks = intel.networks().to_vec();

        div()
            .p_4()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(div().flex_1().child(Input::new(&self.new_network).small()))
                    .child(
                        Button::new("create-network")
                            .primary()
                            .small()
                            .icon(IconName::Plus)
                            .label("Create")
                            .on_click(
                                cx.listener(|this, _, window, cx| this.create_network(window, cx)),
                            ),
                    ),
            )
            .when(networks.is_empty(), |el| {
                el.child(
                    div()
                        .py_6()
                        .text_center()
                        .text_xs()
                        .text_color(theme::color(TEXT_3))
                        .child("No networks yet. Create one to start sharing intel."),
                )
            })
            .children(networks.into_iter().map(|network| {
                let connected = active == Some(network.id);
                let id = network.id;
                div()
                    .id(("network", id as u64))
                    .flex()
                    .items_center()
                    .gap_3()
                    .px_3()
                    .py_2p5()
                    .rounded_md()
                    .border_1()
                    .border_color(theme::color(if connected { CYAN } else { BORDER }))
                    .bg(theme::color(BG_1))
                    .cursor_pointer()
                    .hover(|s| s.bg(theme::color(BG_HOVER)))
                    .on_click(cx.listener(move |this, _, _, cx| this.open_detail(id, cx)))
                    .child(
                        Icon::new(IconName::Network)
                            .size_4()
                            .text_color(theme::color(CYAN)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(
                                div()
                                    .truncate()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(network.name.clone()),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_3()
                                    .text_size(px(10.))
                                    .text_color(theme::color(TEXT_3))
                                    .child(format!(
                                        "{} annotations",
                                        network.entries_count.unwrap_or(0)
                                    ))
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_1()
                                            .child(crate::ui::dot(
                                                theme::color(if connected {
                                                    GREEN
                                                } else {
                                                    TEXT_3
                                                }),
                                                6.,
                                            ))
                                            .child(if connected { "Connected" } else { "Offline" }),
                                    ),
                            ),
                    )
                    .child(
                        Button::new(("toggle-network", id as u64))
                            .xsmall()
                            .map(|b| if connected { b.ghost() } else { b.primary() })
                            .label(if connected { "Disconnect" } else { "Connect" })
                            .on_click(cx.listener(move |_, _, _, cx| {
                                cx.stop_propagation();
                                Stores::get(cx).intel.update(cx, |intel, cx| {
                                    let next =
                                        (intel.active_network_id() != Some(id)).then_some(id);
                                    intel.set_active_network(next, cx)
                                });
                            })),
                    )
                    .child(
                        Button::new(("delete-network", id as u64))
                            .ghost()
                            .xsmall()
                            .icon(IconName::Trash)
                            .tooltip("Delete network")
                            .on_click(move |_, window, cx| {
                                cx.stop_propagation();
                                confirm_delete_network(id, window, cx)
                            }),
                    )
            }))
    }

    fn render_detail(&self, network: NetworkDetail, cx: &mut Context<Self>) -> impl IntoElement {
        let active = Stores::get(cx).intel.read(cx).active_network_id() == Some(network.id);
        let annotations: Vec<Annotation> = network
            .entries
            .iter()
            .filter_map(|e| Annotation::from_entry_detail(e, network.id, &network.name))
            .collect();
        let network_id = network.id;

        let tab_button = |tab: DetailTab, label: &'static str, cx: &mut Context<Self>| {
            Button::new(label)
                .ghost()
                .xsmall()
                .label(label)
                .selected(self.tab == tab)
                .when(self.tab == tab, |b| b.bg(theme::color(BG_3)))
                .on_click(cx.listener(move |this, _, _, cx| this.set_tab(tab, cx)))
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .px_4()
                    .py_3()
                    .border_b_1()
                    .border_color(theme::color(BORDER))
                    .bg(theme::tint(BG_1, 0x80))
                    .child(
                        Button::new("network-back")
                            .ghost()
                            .xsmall()
                            .icon(IconName::ChevronLeft)
                            .tooltip("All networks")
                            .on_click(cx.listener(|this, _, _, cx| this.back(cx))),
                    )
                    .child(
                        div()
                            .size(px(32.))
                            .flex_none()
                            .rounded_sm()
                            .bg(theme::tint(CYAN, 0x1a))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                Icon::new(IconName::Network)
                                    .size_4()
                                    .text_color(theme::color(CYAN)),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(
                                div()
                                    .truncate()
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(network.name.clone()),
                            )
                            .child(
                                div()
                                    .flex()
                                    .gap_3()
                                    .mt_0p5()
                                    .text_size(px(10.))
                                    .text_color(theme::color(TEXT_3))
                                    .child(format!("{} annotations", annotations.len()))
                                    .child(format!("{} members", network.accesses.len())),
                            ),
                    )
                    .child(
                        Button::new("refresh-network")
                            .ghost()
                            .xsmall()
                            .icon(IconName::RefreshCw)
                            .tooltip("Refresh")
                            .on_click(move |_, _, cx| {
                                Stores::get(cx)
                                    .intel
                                    .update(cx, |intel, cx| intel.select_network(network_id, cx))
                            }),
                    )
                    .child(if active {
                        Button::new("disconnect-network")
                            .ghost()
                            .xsmall()
                            .icon(IconName::LogOut)
                            .label("Disconnect")
                            .on_click(|_, _, cx| {
                                Stores::get(cx)
                                    .intel
                                    .update(cx, |intel, cx| intel.set_active_network(None, cx))
                            })
                    } else {
                        Button::new("connect-network")
                            .primary()
                            .xsmall()
                            .label("Connect")
                            .on_click(move |_, _, cx| {
                                Stores::get(cx).intel.update(cx, |intel, cx| {
                                    intel.set_active_network(Some(network_id), cx)
                                })
                            })
                    }),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .py_2()
                    .border_b_1()
                    .border_color(theme::color(BORDER))
                    .child(
                        div()
                            .flex()
                            .gap_1()
                            .child(tab_button(DetailTab::Annotations, "Annotations", cx))
                            .child(tab_button(DetailTab::Scans, "Scans", cx))
                            .child(tab_button(DetailTab::Access, "Access", cx)),
                    )
                    .when(self.tab == DetailTab::Annotations, |el| {
                        el.child(
                            Button::new("add-annotation")
                                .ghost()
                                .xsmall()
                                .icon(IconName::Plus)
                                .label("Add")
                                .text_color(theme::color(CYAN))
                                .on_click(move |_, window, cx| {
                                    annotation_form::open(network_id, None, None, window, cx)
                                }),
                        )
                    })
                    .when(self.tab == DetailTab::Access, |el| {
                        el.child(
                            Button::new("grant-access")
                                .ghost()
                                .xsmall()
                                .icon(IconName::Plus)
                                .label("Grant Access")
                                .text_color(theme::color(CYAN))
                                .on_click(move |_, window, cx| {
                                    open_access_dialog(network_id, window, cx)
                                }),
                        )
                    }),
            )
            .child(
                div()
                    .id("network-tab-content")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .child(match self.tab {
                        DetailTab::Annotations => {
                            entries_tab(network_id, annotations).into_any_element()
                        }
                        DetailTab::Access => {
                            access_tab(network_id, &network.accesses).into_any_element()
                        }
                        DetailTab::Scans => self.scans_tab(cx).into_any_element(),
                    }),
            )
    }

    fn scans_tab(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let now = Utc::now();
        let Some(page) = &self.scans else {
            return div()
                .p_4()
                .text_xs()
                .text_color(theme::color(TEXT_3))
                .child(if self.scans_loading {
                    "Loading scans..."
                } else {
                    "No scans yet"
                })
                .into_any_element();
        };
        let (current, last) = (page.current_page, page.last_page);
        div()
            .flex()
            .flex_col()
            .when(page.data.is_empty(), |el| {
                el.child(
                    div()
                        .p_4()
                        .text_xs()
                        .text_color(theme::color(TEXT_3))
                        .child("No scans yet"),
                )
            })
            .children(page.data.iter().map(|scan| scan_row(scan, now)))
            .when(last > 1, |el| {
                el.child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px_4()
                        .py_2()
                        .text_size(px(10.))
                        .text_color(theme::color(TEXT_3))
                        .child(
                            Button::new("scans-prev")
                                .ghost()
                                .xsmall()
                                .icon(IconName::ChevronLeft)
                                .disabled(current <= 1 || self.scans_loading)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.scans_page -= 1;
                                    this.load_scans(cx);
                                })),
                        )
                        .child(format!("{current} / {last}"))
                        .child(
                            Button::new("scans-next")
                                .ghost()
                                .xsmall()
                                .icon(IconName::ChevronRight)
                                .disabled(current >= last || self.scans_loading)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.scans_page += 1;
                                    this.load_scans(cx);
                                })),
                        ),
                )
            })
            .into_any_element()
    }
}

impl Render for NetworkManager {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let intel = Stores::get(cx).intel.read(cx);
        let authenticated = intel.is_authenticated();
        let selected = intel.selected_network().cloned();

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme::color(BG_0))
            .when_some(self.error.clone(), |el, error| {
                el.child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px_4()
                        .py_2()
                        .bg(theme::tint(RED, 0x1a))
                        .border_b_1()
                        .border_color(theme::tint(RED, 0x33))
                        .text_xs()
                        .text_color(theme::color(RED))
                        .child(error)
                        .child(
                            Button::new("dismiss-error")
                                .ghost()
                                .xsmall()
                                .icon(IconName::X)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.error = None;
                                    cx.notify();
                                })),
                        ),
                )
            })
            .child(
                div()
                    .id("network-content")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .child(if !authenticated {
                        self.render_unauthenticated().into_any_element()
                    } else if let Some(network) = selected.filter(|_| self.detail) {
                        self.render_detail(network, cx).into_any_element()
                    } else {
                        self.render_list(cx).into_any_element()
                    }),
            )
    }
}

fn confirm_delete_network(id: i64, window: &mut Window, cx: &mut App) {
    let name = Stores::get(cx)
        .intel
        .read(cx)
        .networks()
        .iter()
        .find(|n| n.id == id)
        .map(|n| n.name.clone())
        .unwrap_or_default();
    window.open_alert_dialog(cx, move |alert, _, _| {
        alert
            .title(format!("Delete \u{201c}{name}\u{201d}?"))
            .description("Its annotations and scan history are deleted for every member.")
            .confirm()
            .ok_text("Delete")
            .ok_variant(gpui_kit::component::button::ButtonVariant::Danger)
            .on_ok(move |_, _, cx| {
                Stores::get(cx)
                    .intel
                    .update(cx, |intel, cx| intel.delete_network(id, cx));
                true
            })
    });
}

fn entries_tab(network_id: i64, annotations: Vec<Annotation>) -> impl IntoElement {
    if annotations.is_empty() {
        return div()
            .p_6()
            .flex()
            .flex_col()
            .items_center()
            .gap_2()
            .text_xs()
            .text_color(theme::color(TEXT_3))
            .child("No annotations yet")
            .child(
                Button::new("add-first-annotation")
                    .ghost()
                    .xsmall()
                    .icon(IconName::Plus)
                    .label("Add your first annotation")
                    .on_click(move |_, window, cx| {
                        annotation_form::open(network_id, None, None, window, cx)
                    }),
            )
            .into_any_element();
    }
    div()
        .flex()
        .flex_col()
        .children(annotations.into_iter().map(move |annotation| {
            let color = annotation
                .color
                .clone()
                .unwrap_or_else(|| DEFAULT_ANNOTATION_COLOR.into());
            let id = annotation.id;
            let edit = annotation.clone();
            div()
                .id(("annotation", id as u64))
                .flex()
                .items_start()
                .gap_3()
                .px_4()
                .py_2p5()
                .border_b_1()
                .border_color(theme::color(BORDER))
                .hover(|s| s.bg(theme::color(BG_1)))
                .when_some(
                    portrait_url(annotation.target_type.as_str(), annotation.target_id, 64),
                    |el, url| el.child(img(url).size(px(28.)).rounded_sm().flex_none()),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .truncate()
                                        .text_xs()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .child(annotation.target_name.clone()),
                                )
                                .child(
                                    div()
                                        .text_size(px(9.))
                                        .text_color(theme::color(TEXT_3))
                                        .child(annotation.target_type.as_str().to_uppercase()),
                                ),
                        )
                        .when(!annotation.tags.is_empty(), |el| {
                            el.child(div().flex().flex_wrap().gap_1().mt_1().children(
                                annotation.tags.iter().map(|tag| {
                                    div()
                                        .px_1p5()
                                        .rounded_sm()
                                        .text_size(px(10.))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .bg(theme::hex_tint(&color, 0x22, BG_3))
                                        .text_color(theme::hex_or(&color, TEXT_2))
                                        .child(tag.clone())
                                }),
                            ))
                        })
                        .when_some(
                            annotation.note.clone().filter(|n| !n.trim().is_empty()),
                            |el, note| {
                                el.child(
                                    div()
                                        .mt_1()
                                        .text_size(px(11.))
                                        .text_color(theme::color(TEXT_2))
                                        .child(note),
                                )
                            },
                        ),
                )
                .child(
                    Button::new(("edit-annotation", id as u64))
                        .ghost()
                        .xsmall()
                        .icon(IconName::Pencil)
                        .tooltip("Edit")
                        .on_click(move |_, window, cx| {
                            let target = Target {
                                entity_type: edit.target_type,
                                id: edit.target_id,
                                name: edit.target_name.clone(),
                            };
                            annotation_form::open(network_id, Some(target), Some(&edit), window, cx)
                        }),
                )
                .child(
                    Button::new(("delete-annotation", id as u64))
                        .ghost()
                        .xsmall()
                        .icon(IconName::Trash)
                        .tooltip("Delete")
                        .on_click(move |_, _, cx| {
                            Stores::get(cx)
                                .intel
                                .update(cx, |intel, cx| intel.remove_entry(network_id, id, cx))
                        }),
                )
        }))
        .into_any_element()
}

fn access_tab(network_id: i64, accesses: &[NetworkAccess]) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .children(accesses.iter().map(|access| {
            let entity = access.entity.as_ref();
            let name = entity
                .map(|e| e.name.clone())
                .unwrap_or_else(|| access.accessible_id.to_string());
            let ticker = entity.and_then(|e| e.ticker.clone());
            let affiliation = entity.map(|e| {
                [
                    e.corporation.as_ref().map(|c| c.name.clone()),
                    e.alliance.as_ref().map(|a| a.name.clone()),
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(" · ")
            });
            let access_id = access.id;
            let removable = can_remove_access(access);
            div()
                .flex()
                .items_center()
                .gap_3()
                .px_4()
                .py_2()
                .border_b_1()
                .border_color(theme::color(BORDER))
                .when_some(
                    portrait_url(&access.accessible_type, access.accessible_id, 64),
                    |el, url| el.child(img(url).size(px(28.)).rounded_sm().flex_none()),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .child(
                            div()
                                .flex()
                                .gap_1()
                                .text_xs()
                                .child(div().truncate().child(name))
                                .when_some(ticker, |el, t| {
                                    el.child(
                                        div()
                                            .text_color(theme::color(TEXT_3))
                                            .child(format!("[{t}]")),
                                    )
                                }),
                        )
                        .when_some(affiliation.filter(|a| !a.is_empty()), |el, a| {
                            el.child(
                                div()
                                    .truncate()
                                    .text_size(px(10.))
                                    .text_color(theme::color(TEXT_3))
                                    .child(a),
                            )
                        }),
                )
                .child(
                    div()
                        .px_1p5()
                        .rounded_sm()
                        .text_size(px(10.))
                        .bg(theme::color(BG_3))
                        .text_color(theme::color(if access.is_owner { CYAN } else { TEXT_2 }))
                        .child(access_permission_label(access).to_string()),
                )
                .when(removable, |el| {
                    el.child(
                        Button::new(("remove-access", access_id as u64))
                            .ghost()
                            .xsmall()
                            .icon(IconName::Trash)
                            .tooltip("Remove access")
                            .on_click(move |_, _, cx| {
                                Stores::get(cx).intel.update(cx, |intel, cx| {
                                    intel.remove_access(network_id, access_id, cx)
                                })
                            }),
                    )
                })
        }))
}

fn scan_row(scan: &NetworkScan, now: chrono::DateTime<Utc>) -> impl IntoElement {
    let raw = scan.raw_text.clone();
    let dscan = scan.scan_type == "dscan";
    let entries = scan
        .raw_text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .count();
    div()
        .id(("history-scan", scan.id as u64))
        .flex()
        .items_center()
        .gap_3()
        .px_4()
        .py_2()
        .border_b_1()
        .border_color(theme::color(BORDER))
        .cursor_pointer()
        .hover(|s| s.bg(theme::color(BG_HOVER)))
        .on_click(move |_, _, cx| {
            Stores::get(cx).scan.update(cx, |scan, cx| {
                scan.load(&raw, cx);
            })
        })
        .child(
            div()
                .px_1p5()
                .rounded_sm()
                .text_size(px(9.))
                .font_weight(FontWeight::BOLD)
                .bg(theme::tint(if dscan { CYAN } else { GREEN }, 0x1a))
                .text_color(theme::color(if dscan { CYAN } else { GREEN }))
                .child(if dscan { "D-SCAN" } else { "LOCAL" }),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .child(
                    div()
                        .truncate()
                        .text_xs()
                        .text_color(theme::color(TEXT_1))
                        .child(
                            scan.submitted_by
                                .as_ref()
                                .map(|s| s.character_name.clone())
                                .unwrap_or_else(|| "Unknown".into()),
                        ),
                )
                .child(
                    div()
                        .text_size(px(10.))
                        .text_color(theme::color(TEXT_3))
                        .child(
                            [
                                Some(format!("{entries} entries")),
                                scan.solar_system.clone(),
                            ]
                            .into_iter()
                            .flatten()
                            .collect::<Vec<_>>()
                            .join(" · "),
                        ),
                ),
        )
        .child(
            div()
                .text_size(px(10.))
                .text_color(theme::color(TEXT_3))
                .child(relative_time(&scan.created_at, now).unwrap_or_default()),
        )
}

struct AccessForm {
    network_id: i64,
    search: Entity<EntitySearch>,
    target: Option<(String, i64, String)>,
    permission: usize,
    _subscription: Subscription,
}

impl AccessForm {
    fn new(network_id: i64, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| EntitySearch::new(None, window, cx));
        let subscription = cx.subscribe(&search, |this, _, event, cx| {
            let EntitySearchEvent::Selected(result) = event;
            this.target = Some((result.category.clone(), result.id, result.name.clone()));
            cx.notify();
        });
        Self {
            network_id,
            search,
            target: None,
            permission: 0,
            _subscription: subscription,
        }
    }

    fn save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some((kind, id, name)) = self.target.clone() else {
            return;
        };
        let input = AccessInput {
            accessible_type: kind,
            accessible_id: id,
            accessible_name: name,
            permission: PermissionLevel::ALL[self.permission].as_str().to_string(),
        };
        let network_id = self.network_id;
        Stores::get(cx)
            .intel
            .update(cx, |intel, cx| intel.add_access(network_id, input, cx));
        window.close_dialog(cx);
    }
}

impl Render for AccessForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let level = PermissionLevel::ALL[self.permission];
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(self.search.clone())
            .child(
                RadioGroup::horizontal("permission")
                    .children(PermissionLevel::ALL.map(|p| {
                        let label: SharedString = match p {
                            PermissionLevel::Viewer => "Viewer",
                            PermissionLevel::Member => "Member",
                            PermissionLevel::Manager => "Manager",
                        }
                        .into();
                        label
                    }))
                    .selected_index(Some(self.permission))
                    .on_change(cx.listener(|this, index: &usize, _, cx| {
                        this.permission = *index;
                        cx.notify();
                    })),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(theme::color(TEXT_3))
                    .child(describe_permission(level)),
            )
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap_2()
                    .child(
                        Button::new("access-cancel")
                            .ghost()
                            .label("Cancel")
                            .on_click(|_, window, cx| window.close_dialog(cx)),
                    )
                    .child(
                        Button::new("access-save")
                            .primary()
                            .label("Grant")
                            .disabled(self.target.is_none())
                            .on_click(cx.listener(|this, _, window, cx| this.save(window, cx))),
                    ),
            )
    }
}

fn open_access_dialog(network_id: i64, window: &mut Window, cx: &mut App) {
    let form = cx.new(|cx| AccessForm::new(network_id, window, cx));
    window.open_dialog(cx, move |dialog, _, _| {
        dialog.title("Grant access").w(px(420.)).child(form.clone())
    });
}
