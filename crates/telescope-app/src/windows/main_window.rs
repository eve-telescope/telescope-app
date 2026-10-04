use gpui_kit::component::TitleBar;
use gpui_kit::{
    AnyWindowHandle, App, AppContext as _, Bounds, Context, Entity, Global,
    InteractiveElement as _, IntoElement, ParentElement as _, Render,
    StatefulInteractiveElement as _, Styled as _, Subscription, Window, WindowBounds,
    WindowOptions, div, point, px, size,
};
use telescope_core::view::scan_input::ScanInputKind;

use crate::state::Stores;
use crate::state::scan::ScanEvent;
use crate::theme::{self, BG_0, CYAN, TEXT_1, TEXT_2, TEXT_3};
use crate::ui::{Icon, IconName};
use crate::views::dscan_panel::DscanPanel;
use crate::views::input_panel::InputPanel;
use crate::views::local_panel::LocalPanel;
use crate::windows::{overlay, settings_window};

/// Which results fill the main window; follows the kind of the last scan.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Results {
    Local,
    Dscan,
}

struct MainWindow(AnyWindowHandle, Entity<MainView>);

impl Global for MainWindow {}

pub struct MainView {
    results: Results,
    input: Entity<InputPanel>,
    local: Entity<LocalPanel>,
    dscan: Entity<DscanPanel>,
    _subscriptions: Vec<Subscription>,
}

impl MainView {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let stores = Stores::get(cx);
        let subscriptions = vec![
            cx.subscribe(&stores.scan, |this, _, event, cx| match event {
                ScanEvent::Started(ScanInputKind::Local) => this.show(Results::Local, cx),
                ScanEvent::Started(ScanInputKind::Dscan) => this.show(Results::Dscan, cx),
            }),
            cx.observe(&stores.scan, |_, _, cx| cx.notify()),
            cx.observe_window_bounds(window, |_, window, cx| save_bounds(window, cx)),
        ];
        Self {
            results: Results::Local,
            input: cx.new(|cx| InputPanel::new(window, cx)),
            local: cx.new(|cx| LocalPanel::new(window, cx)),
            dscan: cx.new(|cx| DscanPanel::new(window, cx)),
            _subscriptions: subscriptions,
        }
    }

    fn show(&mut self, results: Results, cx: &mut Context<Self>) {
        if self.results != results {
            self.results = results;
            cx.notify();
        }
    }

    fn render_title_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let pilot_count = Stores::get(cx).scan.read(cx).pilots().len();
        let overlay_open = overlay::is_open(cx);

        let overlay_toggle = title_button(
            "overlay-toggle",
            IconName::Layers,
            if overlay_open {
                "Close overlay"
            } else {
                "Open overlay"
            },
            overlay_open,
        )
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
            .child(div().flex_1())
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .mr_2()
                    .child(overlay_toggle)
                    .child(
                        title_button("open-settings", IconName::Settings, "Settings", false)
                            .on_click(|_, _, cx| settings_window::open(cx)),
                    ),
            )
    }
}

impl Render for MainView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content = match self.results {
            Results::Local => self.local.clone().into_any_element(),
            Results::Dscan => self.dscan.clone().into_any_element(),
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

/// An icon button for the title bar, highlighted while `active`.
fn title_button(
    id: &'static str,
    icon: IconName,
    tooltip: &'static str,
    active: bool,
) -> gpui_kit::Stateful<gpui_kit::Div> {
    div()
        .id(id)
        .size(px(30.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(7.))
        .cursor_pointer()
        .map(|el| {
            if active {
                el.bg(theme::tint(CYAN, 0x26))
                    .text_color(theme::color(CYAN))
            } else {
                el.text_color(theme::color(TEXT_2))
                    .hover(|s| s.bg(theme::hairline(0x14)).text_color(theme::color(TEXT_1)))
            }
        })
        .tooltip(move |window, cx| {
            gpui_kit::component::tooltip::Tooltip::new(tooltip).build(window, cx)
        })
        .child(Icon::new(icon).size(px(18.)))
}

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
                if !cx.windows().contains(&main) {
                    cx.quit();
                }
            })
            .detach();
        }
        Err(e) => log::error!("Failed to open main window: {}", e),
    }
}

pub fn handle(cx: &App) -> Option<AnyWindowHandle> {
    cx.try_global::<MainWindow>().map(|w| w.0)
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
