use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{Selectable as _, Sizable as _, TitleBar};
use gpui_kit::{
    AnyWindowHandle, App, AppContext as _, Bounds, Context, Entity, Global,
    InteractiveElement as _, IntoElement, ParentElement as _, Render, SharedString,
    StatefulInteractiveElement as _, Styled as _, Subscription, Window, WindowBounds,
    WindowOptions, div, point, px, size,
};
use telescope_core::view::scan_input::ScanInputKind;

use crate::state::Stores;
use crate::state::scan::ScanEvent;
use crate::theme::{self, BG_0, BG_3, BG_HOVER, CYAN, TEXT_1, TEXT_3};
use crate::ui::{Icon, IconName};
use crate::views::dscan_panel::DscanPanel;
use crate::views::input_panel::InputPanel;
use crate::views::local_panel::LocalPanel;
use crate::views::network_manager::NetworkManager;
use crate::views::settings_panel::SettingsPanel;
use crate::windows::overlay;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Local,
    Dscan,
    Networks,
    Settings,
}

impl Tab {
    const ALL: [Tab; 4] = [Tab::Local, Tab::Dscan, Tab::Networks, Tab::Settings];

    fn label(self) -> &'static str {
        match self {
            Tab::Local => "Local",
            Tab::Dscan => "D-Scan",
            Tab::Networks => "Network",
            Tab::Settings => "Settings",
        }
    }
}

struct MainWindow(AnyWindowHandle, Entity<MainView>);

impl Global for MainWindow {}

pub struct MainView {
    tab: Tab,
    input: Entity<InputPanel>,
    local: Entity<LocalPanel>,
    dscan: Entity<DscanPanel>,
    networks: Entity<NetworkManager>,
    settings: Entity<SettingsPanel>,
    _subscriptions: Vec<Subscription>,
}

impl MainView {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let stores = Stores::get(cx);
        let subscriptions = vec![
            cx.subscribe(&stores.scan, |this, _, event, cx| match event {
                ScanEvent::Started(ScanInputKind::Local) => this.set_tab(Tab::Local, cx),
                ScanEvent::Started(ScanInputKind::Dscan) => this.set_tab(Tab::Dscan, cx),
            }),
            cx.observe(&stores.scan, |_, _, cx| cx.notify()),
            cx.observe_window_bounds(window, |_, window, cx| save_bounds(window, cx)),
        ];
        Self {
            tab: Tab::Local,
            input: cx.new(|cx| InputPanel::new(window, cx)),
            local: cx.new(|cx| LocalPanel::new(window, cx)),
            dscan: cx.new(|cx| DscanPanel::new(window, cx)),
            networks: cx.new(|cx| NetworkManager::new(window, cx)),
            settings: cx.new(|cx| SettingsPanel::new(window, cx)),
            _subscriptions: subscriptions,
        }
    }

    pub fn set_tab(&mut self, tab: Tab, cx: &mut Context<Self>) {
        if self.tab != tab {
            self.tab = tab;
            cx.notify();
        }
    }

    fn render_title_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let pilot_count = Stores::get(cx).scan.read(cx).pilots().len();
        let overlay_open = overlay::is_open(cx);

        let nav = div()
            .flex()
            .items_center()
            .gap_1()
            .children(Tab::ALL.map(|tab| {
                let active = self.tab == tab;
                Button::new(SharedString::from(tab.label()))
                    .ghost()
                    .xsmall()
                    .label(tab.label())
                    .selected(active)
                    .text_size(px(10.))
                    .text_color(theme::color(if active { TEXT_1 } else { TEXT_3 }))
                    .when(active, |el| el.bg(theme::color(BG_3)))
                    .on_click(cx.listener(move |this, _, _, cx| this.set_tab(tab, cx)))
            }));

        let overlay_toggle = Button::new("overlay-toggle")
            .ghost()
            .xsmall()
            .mr_2()
            .icon(Icon::new(IconName::Layers))
            .label("Overlay")
            .tooltip("Toggle overlay window")
            .selected(overlay_open)
            .text_size(px(10.))
            .map(|el| {
                if overlay_open {
                    el.bg(theme::tint(CYAN, 0x33))
                        .text_color(theme::color(CYAN))
                } else {
                    el.text_color(theme::color(TEXT_3))
                }
            })
            .on_click(|_, _, cx| overlay::toggle(cx));

        TitleBar::new()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        Icon::new(IconName::Crosshair)
                            .size_4()
                            .text_color(theme::color(CYAN)),
                    )
                    .child(
                        div()
                            .text_size(px(10.))
                            .font_weight(gpui_kit::FontWeight::BOLD)
                            .text_color(theme::color(TEXT_3))
                            .child("TELESCOPE"),
                    )
                    .when(pilot_count > 0, |el| {
                        el.child(
                            div()
                                .px_1p5()
                                .rounded_sm()
                                .bg(theme::tint(CYAN, 0x1a))
                                .text_size(px(9.))
                                .text_color(theme::color(CYAN))
                                .font_family("monospace")
                                .child(pilot_count.to_string()),
                        )
                    }),
            )
            .child(div().flex_1().flex().justify_center().child(nav))
            .child(overlay_toggle)
    }
}

impl Render for MainView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content = match self.tab {
            Tab::Local => self.local.clone().into_any_element(),
            Tab::Dscan => self.dscan.clone().into_any_element(),
            Tab::Networks => self.networks.clone().into_any_element(),
            Tab::Settings => self.settings.clone().into_any_element(),
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme::color(BG_0))
            .text_color(theme::color(TEXT_1))
            .text_size(px(13.))
            .child(self.render_title_bar(cx))
            .child(
                div()
                    .flex_1()
                    .flex()
                    .min_h_0()
                    .overflow_hidden()
                    .child(self.input.clone())
                    .child(div().flex_1().min_w_0().h_full().child(content)),
            )
    }
}

use gpui_kit::prelude::FluentBuilder as _;

fn save_bounds(window: &mut Window, cx: &mut App) {
    if let WindowBounds::Windowed(bounds) = window.window_bounds() {
        let stores = Stores::get(cx);
        stores.settings.update(cx, |settings, cx| {
            settings.update(cx, |s| {
                s.main_window = Some(telescope_core::settings::WindowBounds {
                    x: bounds.origin.x.into(),
                    y: bounds.origin.y.into(),
                    width: bounds.size.width.into(),
                    height: bounds.size.height.into(),
                })
            })
        });
    }
}

pub fn open(cx: &mut App) {
    let saved = Stores::get(cx).settings.read(cx).get().main_window;
    let bounds = match saved {
        Some(b) => Bounds::new(point(px(b.x), px(b.y)), size(px(b.width), px(b.height))),
        None => Bounds::centered(None, size(px(1500.), px(800.)), cx),
    };
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        window_min_size: Some(size(px(1350.), px(600.))),
        app_id: Some(telescope_core::config::IDENTIFIER.to_string()),
        ..TitleBar::window_options()
    };
    match gpui_kit::open_window(options, cx, |window, cx| {
        window.set_window_title("Telescope");
        cx.new(|cx| MainView::new(window, cx))
    }) {
        Ok((handle, view)) => {
            cx.set_global(MainWindow(handle, view));
            cx.on_window_closed(|cx, _| {
                if !cx.has_global::<MainWindow>() {
                    return;
                }
                let main = cx.global::<MainWindow>().0;
                if !cx.windows().iter().any(|w| *w == main) {
                    cx.quit();
                }
            })
            .detach();
        }
        Err(e) => log::error!("Failed to open main window: {}", e),
    }
}

pub fn focus(cx: &mut App) {
    if let Some(MainWindow(handle, _)) = cx.try_global::<MainWindow>() {
        let handle = *handle;
        let _ = handle.update(cx, |_, window, _| window.activate_window());
    }
}

/// Runs a scan as if it had been pasted into the input panel.
pub fn scan(text: &str, cx: &mut App) {
    if let Some(MainWindow(handle, view)) = cx.try_global::<MainWindow>() {
        let (handle, input) = (*handle, view.read(cx).input.clone());
        let _ = handle.update(cx, |_, window, cx| {
            input.update(cx, |input, cx| input.set_text(text, window, cx))
        });
    }
    Stores::get(cx)
        .scan
        .update(cx, |scan, cx| scan.submit(text, cx));
}
