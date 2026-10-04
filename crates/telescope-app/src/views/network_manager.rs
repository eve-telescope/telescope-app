//! The Intel Network settings page: the list of networks, and a detail view
//! with annotations, shared scans and members.

use chrono::Utc;
use gpui_kit::component::button::{Button, ButtonVariant, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::radio::RadioGroup;
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{Disableable as _, Sizable as _, WindowExt as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AnyElement, App, AppContext as _, Context, Entity, FontWeight, InteractiveElement as _,
    IntoElement, ParentElement as _, Render, SharedString, StatefulInteractiveElement as _,
    Styled as _, Subscription, Task, Window, div, px,
};
use telescope_core::intel_service::AccessInput;
use telescope_core::models::{NetworkAccess, NetworkDetail, NetworkScan, PaginatedScans};
use telescope_core::view::annotations::{Annotation, DEFAULT_ANNOTATION_COLOR, EntityType, Target};
use telescope_core::view::format::relative_time;
use telescope_core::view::network::{
    PermissionLevel, access_permission_label, can_remove_access, describe_permission,
};

use crate::state::Stores;
use crate::state::intel::IntelEvent;
use crate::theme::{self, BG_0, CYAN, GREEN, RED, TEXT_1, TEXT_2, TEXT_3};
use crate::ui::{Icon, IconName};
use crate::views::annotation_form;
use crate::views::entity_search::{EntitySearch, EntitySearchEvent};
use crate::views::intel_card::{scope_pill, target_avatar};
use crate::views::settings_ui::{empty_row, group, page, page_body, row};

#[derive(Clone, Copy, PartialEq)]
enum DetailTab {
    Annotations,
    Scans,
    Members,
}

impl DetailTab {
    const ALL: [DetailTab; 3] = [DetailTab::Annotations, DetailTab::Scans, DetailTab::Members];

    fn label(self) -> &'static str {
        match self {
            DetailTab::Annotations => "Annotations",
            DetailTab::Scans => "Scans",
            DetailTab::Members => "Members",
        }
    }
}

pub struct NetworkManager {
    detail: bool,
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
            cx.subscribe_in(
                &new_network,
                window,
                |this, _, event, window, cx| match event {
                    InputEvent::PressEnter { .. } => this.create_network(window, cx),
                    InputEvent::Change => cx.notify(),
                    _ => {}
                },
            ),
            cx.observe(&stores.intel, |this, intel, cx| {
                if !intel.read(cx).is_authenticated() {
                    this.detail = false;
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

    fn render_signed_out(&self) -> impl IntoElement {
        page(
            "Intel networks",
            "Share scans and pilot annotations with your corp or alliance.",
        )
        .child(group(
            None,
            vec![
                row(
                    "Not connected",
                    Some("Connect your EVE character under General to use intel networks.".into()),
                )
                .into_any_element(),
            ],
        ))
    }

    fn render_list(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let intel = Stores::get(cx).intel.read(cx);
        let active = intel.active_network_id();
        let networks = intel.networks().to_vec();
        let name_empty = self.new_network.read(cx).value().trim().is_empty();

        let mut rows: Vec<AnyElement> = networks
            .into_iter()
            .map(|network| {
                let id = network.id;
                let is_active = active == Some(id);
                div()
                    .id(("network", id as u64))
                    .flex()
                    .items_center()
                    .gap_3()
                    .px_4()
                    .py_3()
                    .cursor_pointer()
                    .hover(|s| s.bg(theme::hairline(0x08)))
                    .on_click(cx.listener(move |this, _, _, cx| this.open_detail(id, cx)))
                    .child(network_icon(is_active))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .gap_0p5()
                            .child(
                                div()
                                    .truncate()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(theme::color(TEXT_1))
                                    .child(network.name.clone()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::color(TEXT_3))
                                    .child(annotation_count(network.entries_count.unwrap_or(0))),
                            ),
                    )
                    .child(active_switch(id, is_active))
                    .child(
                        Icon::new(IconName::ChevronRight)
                            .size_4()
                            .text_color(theme::color(TEXT_3)),
                    )
                    .into_any_element()
            })
            .collect();
        if rows.is_empty() {
            rows.push(empty_row(
                "No networks yet. Create one below to start sharing intel.",
            ));
        }

        page(
            "Intel networks",
            "The active network receives your scans and supplies the annotations shown on pilots.",
        )
        .child(group(Some("Networks"), rows))
        .child(group(
            Some("New network"),
            vec![
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_4()
                    .py_3()
                    .child(div().flex_1().child(Input::new(&self.new_network).small()))
                    .child(
                        Button::new("create-network")
                            .primary()
                            .small()
                            .label("Create")
                            .disabled(name_empty)
                            .on_click(
                                cx.listener(|this, _, window, cx| this.create_network(window, cx)),
                            ),
                    )
                    .into_any_element(),
            ],
        ))
    }

    fn render_detail(&self, network: NetworkDetail, cx: &mut Context<Self>) -> impl IntoElement {
        let is_active = Stores::get(cx).intel.read(cx).active_network_id() == Some(network.id);
        let annotations: Vec<Annotation> = network
            .entries
            .iter()
            .filter_map(|e| Annotation::from_entry_detail(e, network.id, &network.name))
            .collect();
        let network_id = network.id;
        let counts = [
            annotations.len(),
            self.scans.as_ref().map_or(0, |s| s.data.len()),
            network.accesses.len(),
        ];

        let header = div()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .id("network-back")
                    .flex()
                    .items_center()
                    .gap_1()
                    .text_xs()
                    .text_color(theme::color(TEXT_3))
                    .cursor_pointer()
                    .hover(|s| s.text_color(theme::color(TEXT_1)))
                    .on_click(cx.listener(|this, _, _, cx| this.back(cx)))
                    .child(Icon::new(IconName::ChevronLeft).size_3p5())
                    .child("Intel networks"),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(network_icon(is_active))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(
                                div()
                                    .truncate()
                                    .text_xl()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme::color(TEXT_1))
                                    .child(network.name.clone()),
                            )
                            .child(div().text_xs().text_color(theme::color(TEXT_3)).child(
                                format!(
                                    "{} · {} members",
                                    annotation_count(annotations.len() as i64),
                                    network.accesses.len()
                                ),
                            )),
                    )
                    .child(
                        Button::new("refresh-network")
                            .ghost()
                            .small()
                            .icon(IconName::RefreshCw)
                            .tooltip("Refresh")
                            .on_click(move |_, _, cx| {
                                Stores::get(cx)
                                    .intel
                                    .update(cx, |intel, cx| intel.select_network(network_id, cx))
                            }),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .text_xs()
                            .text_color(theme::color(TEXT_2))
                            .child("Active")
                            .child(active_switch(network_id, is_active)),
                    ),
            );

        let tabs = div()
            .flex()
            .items_center()
            .gap_5()
            .border_b_1()
            .border_color(theme::hairline(0x14))
            .children(DetailTab::ALL.into_iter().zip(counts).map(|(tab, count)| {
                let selected = self.tab == tab;
                div()
                    .id(SharedString::from(format!("tab-{}", tab.label())))
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .pb_2()
                    .border_b_2()
                    .cursor_pointer()
                    .text_sm()
                    .map(|el| {
                        if selected {
                            el.border_color(theme::color(CYAN))
                                .text_color(theme::color(TEXT_1))
                                .font_weight(FontWeight::MEDIUM)
                        } else {
                            el.border_color(gpui_kit::transparent_black())
                                .text_color(theme::color(TEXT_3))
                                .hover(|s| s.text_color(theme::color(TEXT_2)))
                        }
                    })
                    .on_click(cx.listener(move |this, _, _, cx| this.set_tab(tab, cx)))
                    .child(tab.label())
                    .when(tab != DetailTab::Scans || self.scans.is_some(), |el| {
                        el.child(
                            div()
                                .px_1p5()
                                .rounded_full()
                                .bg(theme::hairline(0x0f))
                                .text_size(px(10.))
                                .text_color(theme::color(TEXT_3))
                                .child(count.to_string()),
                        )
                    })
            }));

        let action = match self.tab {
            DetailTab::Annotations => Some(
                Button::new("add-annotation")
                    .outline()
                    .small()
                    .icon(IconName::Plus)
                    .label("Add annotation")
                    .on_click(move |_, window, cx| {
                        annotation_form::open(network_id, None, None, window, cx)
                    }),
            ),
            DetailTab::Members => Some(
                Button::new("grant-access")
                    .outline()
                    .small()
                    .icon(IconName::Plus)
                    .label("Grant access")
                    .on_click(move |_, window, cx| open_access_dialog(network_id, window, cx)),
            ),
            DetailTab::Scans => None,
        };

        let list = match self.tab {
            DetailTab::Annotations => group(None, annotation_rows(network_id, annotations)),
            DetailTab::Members => group(None, member_rows(network_id, &network.accesses)),
            DetailTab::Scans => self.scans_group(cx),
        };

        page_body()
            .child(header)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(tabs)
                    .when_some(action, |el, action| {
                        el.child(div().flex().justify_end().child(action))
                    })
                    .child(list),
            )
            .child(group(
                Some("Danger zone"),
                vec![
                    row(
                        "Delete network",
                        Some("Removes its annotations and scan history for every member.".into()),
                    )
                    .child(
                        Button::new("delete-network")
                            .with_variant(ButtonVariant::Danger)
                            .small()
                            .label("Delete")
                            .on_click(move |_, window, cx| {
                                confirm_delete_network(network_id, window, cx)
                            }),
                    )
                    .into_any_element(),
                ],
            ))
    }

    fn scans_group(&self, cx: &mut Context<Self>) -> gpui_kit::Div {
        let Some(page) = &self.scans else {
            return group(
                None,
                vec![empty_row(if self.scans_loading {
                    "Loading scans…"
                } else {
                    "No scans yet"
                })],
            );
        };
        let now = Utc::now();
        let mut rows: Vec<AnyElement> = page.data.iter().map(|scan| scan_row(scan, now)).collect();
        if rows.is_empty() {
            rows.push(empty_row("No scans shared to this network yet"));
        }
        let (current, last) = (page.current_page, page.last_page);
        if last > 1 {
            rows.push(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .py_2()
                    .text_xs()
                    .text_color(theme::color(TEXT_3))
                    .child(
                        Button::new("scans-prev")
                            .ghost()
                            .small()
                            .icon(IconName::ChevronLeft)
                            .label("Newer")
                            .disabled(current <= 1 || self.scans_loading)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.scans_page -= 1;
                                this.load_scans(cx);
                            })),
                    )
                    .child(format!("Page {current} of {last}"))
                    .child(
                        Button::new("scans-next")
                            .ghost()
                            .small()
                            .label("Older")
                            .icon(IconName::ChevronRight)
                            .disabled(current >= last || self.scans_loading)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.scans_page += 1;
                                this.load_scans(cx);
                            })),
                    )
                    .into_any_element(),
            );
        }
        group(None, rows)
    }
}

impl Render for NetworkManager {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let intel = Stores::get(cx).intel.read(cx);
        let authenticated = intel.is_authenticated();
        let selected = intel.selected_network().cloned();

        let content = if !authenticated {
            self.render_signed_out().into_any_element()
        } else if let Some(network) = selected.filter(|_| self.detail) {
            self.render_detail(network, cx).into_any_element()
        } else {
            self.render_list(cx).into_any_element()
        };

        div()
            .id("network-settings")
            .size_full()
            .overflow_y_scroll()
            .bg(theme::color(BG_0))
            .when_some(self.error.clone(), |el, error| {
                el.child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .mx_8()
                        .mt_4()
                        .px_3()
                        .py_2()
                        .rounded_md()
                        .bg(theme::tint(RED, 0x1a))
                        .text_xs()
                        .text_color(theme::color(RED))
                        .child(Icon::new(IconName::CircleAlert).size_3p5())
                        .child(div().flex_1().child(error))
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
            .child(content)
    }
}

fn annotation_count(count: i64) -> String {
    match count {
        1 => "1 annotation".to_string(),
        n => format!("{n} annotations"),
    }
}

fn network_icon(active: bool) -> impl IntoElement {
    div()
        .size(px(32.))
        .flex_none()
        .rounded(px(8.))
        .flex()
        .items_center()
        .justify_center()
        .bg(if active {
            theme::tint(CYAN, 0x1f)
        } else {
            theme::hairline(0x0a)
        })
        .child(
            Icon::new(IconName::Network)
                .size_4()
                .text_color(theme::color(if active { CYAN } else { TEXT_3 })),
        )
}

/// Makes this network the active one, or clears it.
fn active_switch(network_id: i64, active: bool) -> impl IntoElement {
    div()
        .on_mouse_down(gpui_kit::MouseButton::Left, |_, _, cx| {
            cx.stop_propagation()
        })
        .child(
            Switch::new(("active-network", network_id as u64))
                .small()
                .checked(active)
                .tooltip(if active {
                    "Active network"
                } else {
                    "Make active"
                })
                .on_change(move |checked: &bool, _, cx| {
                    let next = checked.then_some(network_id);
                    Stores::get(cx)
                        .intel
                        .update(cx, |intel, cx| intel.set_active_network(next, cx))
                }),
        )
}

fn hover_action(
    id: (&'static str, u64),
    icon: IconName,
    tooltip: &'static str,
    group: SharedString,
) -> gpui_kit::Stateful<gpui_kit::Div> {
    div()
        .id(id)
        .flex_none()
        .p_1()
        .rounded_sm()
        .text_color(theme::color(TEXT_3))
        .invisible()
        .group_hover(group, |s| s.visible())
        .cursor_pointer()
        .hover(|s| s.bg(theme::hairline(0x14)).text_color(theme::color(TEXT_1)))
        .tooltip(move |window, cx| {
            gpui_kit::component::tooltip::Tooltip::new(tooltip).build(window, cx)
        })
        .child(Icon::new(icon).size_3p5())
}

fn annotation_rows(network_id: i64, annotations: Vec<Annotation>) -> Vec<AnyElement> {
    if annotations.is_empty() {
        return vec![empty_row(
            "No annotations yet. Right-click a pilot in a scan to tag them.",
        )];
    }
    annotations
        .into_iter()
        .map(|annotation| {
            let color = annotation
                .color
                .clone()
                .unwrap_or_else(|| DEFAULT_ANNOTATION_COLOR.into());
            let id = annotation.id;
            let group: SharedString = format!("annotation-{id}").into();
            let edit = annotation.clone();
            div()
                .group(group.clone())
                .flex()
                .items_start()
                .gap_3()
                .px_4()
                .py_3()
                .hover(|s| s.bg(theme::hairline(0x05)))
                .child(target_avatar(&Target::of(&annotation), 28.))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .truncate()
                                        .text_sm()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(theme::color(TEXT_1))
                                        .child(annotation.target_name.clone()),
                                )
                                .child(scope_pill(annotation.target_type)),
                        )
                        .when(!annotation.tags.is_empty(), |el| {
                            el.child(div().flex().flex_wrap().gap_1().children(
                                annotation.tags.iter().map(|tag| {
                                    div()
                                        .px_2()
                                        .py(px(2.))
                                        .rounded_full()
                                        .text_size(px(10.))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .bg(theme::hex_tint(&color, 0x2e, TEXT_3))
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
                                        .text_xs()
                                        .text_color(theme::color(TEXT_2))
                                        .line_clamp(2)
                                        .child(note),
                                )
                            },
                        ),
                )
                .child(
                    hover_action(
                        ("edit-annotation", id as u64),
                        IconName::Pencil,
                        "Edit",
                        group.clone(),
                    )
                    .on_click(move |_, window, cx| {
                        annotation_form::open(
                            network_id,
                            Some(Target::of(&edit)),
                            Some(&edit),
                            window,
                            cx,
                        )
                    }),
                )
                .child(
                    hover_action(
                        ("delete-annotation", id as u64),
                        IconName::Trash,
                        "Delete",
                        group,
                    )
                    .on_click(move |_, _, cx| {
                        Stores::get(cx)
                            .intel
                            .update(cx, |intel, cx| intel.remove_entry(network_id, id, cx))
                    }),
                )
                .into_any_element()
        })
        .collect()
}

fn member_rows(network_id: i64, accesses: &[NetworkAccess]) -> Vec<AnyElement> {
    if accesses.is_empty() {
        return vec![empty_row("No members")];
    }
    accesses
        .iter()
        .map(|access| {
            let entity = access.entity.as_ref();
            let name = entity
                .map(|e| e.name.clone())
                .unwrap_or_else(|| access.accessible_id.to_string());
            let detail = entity
                .map(|e| {
                    [
                        e.ticker.as_ref().map(|t| format!("[{t}]")),
                        e.corporation.as_ref().map(|c| c.name.clone()),
                        e.alliance.as_ref().map(|a| a.name.clone()),
                    ]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .join(" · ")
                })
                .filter(|d| !d.is_empty());
            let scope = EntityType::parse(
                access
                    .accessible_type
                    .rsplit('\\')
                    .next()
                    .unwrap_or_default()
                    .to_lowercase()
                    .as_str(),
            );
            let target = Target {
                entity_type: scope.unwrap_or(EntityType::Character),
                id: access.accessible_id,
                name: name.clone(),
            };
            let access_id = access.id;
            let group: SharedString = format!("member-{access_id}").into();
            let removable = can_remove_access(access);
            div()
                .group(group.clone())
                .flex()
                .items_center()
                .gap_3()
                .px_4()
                .py_2p5()
                .hover(|s| s.bg(theme::hairline(0x05)))
                .child(target_avatar(&target, 28.))
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
                                        .text_sm()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(theme::color(TEXT_1))
                                        .child(name),
                                )
                                .when_some(scope, |el, scope| el.child(scope_pill(scope))),
                        )
                        .when_some(detail, |el, detail| {
                            el.child(
                                div()
                                    .truncate()
                                    .text_xs()
                                    .text_color(theme::color(TEXT_3))
                                    .child(detail),
                            )
                        }),
                )
                .child(
                    div()
                        .px_2()
                        .py(px(2.))
                        .rounded_full()
                        .text_size(px(10.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .bg(if access.is_owner {
                            theme::tint(CYAN, 0x1f)
                        } else {
                            theme::hairline(0x0f)
                        })
                        .text_color(theme::color(if access.is_owner { CYAN } else { TEXT_2 }))
                        .child(access_permission_label(access).to_string()),
                )
                .when(removable, |el| {
                    el.child(
                        hover_action(
                            ("remove-access", access_id as u64),
                            IconName::Trash,
                            "Remove",
                            group,
                        )
                        .on_click(move |_, _, cx| {
                            Stores::get(cx).intel.update(cx, |intel, cx| {
                                intel.remove_access(network_id, access_id, cx)
                            })
                        }),
                    )
                })
                .into_any_element()
        })
        .collect()
}

fn scan_row(scan: &NetworkScan, now: chrono::DateTime<Utc>) -> AnyElement {
    let raw = scan.raw_text.clone();
    let dscan = scan.scan_type == "dscan";
    let entries = scan
        .raw_text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .count();
    let accent = if dscan { CYAN } else { GREEN };
    div()
        .id(("history-scan", scan.id as u64))
        .flex()
        .items_center()
        .gap_3()
        .px_4()
        .py_2p5()
        .cursor_pointer()
        .hover(|s| s.bg(theme::hairline(0x08)))
        .on_click(move |_, _, cx| {
            Stores::get(cx).scan.update(cx, |scan, cx| {
                scan.load(&raw, cx);
            });
            crate::windows::main_window::focus(cx);
        })
        .child(
            div()
                .size(px(28.))
                .flex_none()
                .rounded(px(7.))
                .flex()
                .items_center()
                .justify_center()
                .bg(theme::tint(accent, 0x1f))
                .child(
                    Icon::new(if dscan {
                        IconName::Radar
                    } else {
                        IconName::Users
                    })
                    .size_3p5()
                    .text_color(theme::color(accent)),
                ),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .child(
                    div()
                        .truncate()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme::color(TEXT_1))
                        .child(
                            scan.submitted_by
                                .as_ref()
                                .map(|s| s.character_name.clone())
                                .unwrap_or_else(|| "Unknown".into()),
                        ),
                )
                .child(
                    div().text_xs().text_color(theme::color(TEXT_3)).child(
                        [
                            Some(if dscan {
                                "D-scan".to_string()
                            } else {
                                "Local".to_string()
                            }),
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
                .text_xs()
                .text_color(theme::color(TEXT_3))
                .child(relative_time(&scan.created_at, now).unwrap_or_default()),
        )
        .child(
            Icon::new(IconName::ChevronRight)
                .size_3p5()
                .text_color(theme::color(TEXT_3)),
        )
        .into_any_element()
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
            .ok_variant(ButtonVariant::Danger)
            .on_ok(move |_, _, cx| {
                Stores::get(cx)
                    .intel
                    .update(cx, |intel, cx| intel.delete_network(id, cx));
                true
            })
    });
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
        dialog.title("Grant access").w(px(440.)).child(form.clone())
    });
}
