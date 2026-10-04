//! The settings window: general settings and the intel network manager,
//! picked from a sidebar.

use gpui_kit::component::TitleBar;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AnyWindowHandle, App, AppContext as _, Bounds, Context, Entity, FontWeight, Global,
    InteractiveElement as _, IntoElement, ParentElement as _, Render, SharedString,
    StatefulInteractiveElement as _, Styled as _, Window, WindowBounds, WindowOptions, div, px,
    size,
};

use crate::theme::{self, BG_0, BG_1, BG_HOVER, BORDER, CYAN, TEXT_1, TEXT_2, TEXT_3};
use crate::ui::{Icon, IconName};
use crate::views::motion;
use crate::views::network_manager::NetworkManager;
use crate::views::settings_panel::SettingsPanel;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Section {
    General,
    Network,
}

impl Section {
    const ALL: [Section; 2] = [Section::General, Section::Network];

    fn label(self) -> &'static str {
        match self {
            Section::General => "General",
            Section::Network => "Intel Network",
        }
    }

    fn icon(self) -> IconName {
        match self {
            Section::General => IconName::Settings,
            Section::Network => IconName::Network,
        }
    }
}

struct SettingsWindow(AnyWindowHandle);

impl Global for SettingsWindow {}

struct SettingsView {
    section: Section,
    general: Entity<SettingsPanel>,
    network: Entity<NetworkManager>,
}

impl SettingsView {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            section: Section::General,
            general: cx.new(|cx| SettingsPanel::new(window, cx)),
            network: cx.new(|cx| NetworkManager::new(window, cx)),
        }
    }

    fn render_nav(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w(px(200.))
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .gap_0p5()
            .p_2()
            .bg(theme::color(BG_1))
            .border_r_1()
            .border_color(theme::color(BORDER))
            .children(Section::ALL.map(|section| {
                let active = self.section == section;
                div()
                    .id(SharedString::from(section.label()))
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_2p5()
                    .py_1p5()
                    .rounded_md()
                    .text_sm()
                    .cursor_pointer()
                    .map(|el| {
                        if active {
                            el.bg(theme::tint(CYAN, 0x1a))
                                .text_color(theme::color(TEXT_1))
                                .font_weight(FontWeight::MEDIUM)
                        } else {
                            el.text_color(theme::color(TEXT_2))
                                .hover(|s| s.bg(theme::color(BG_HOVER)))
                        }
                    })
                    .child(
                        Icon::new(section.icon())
                            .size_4()
                            .text_color(theme::color(if active { CYAN } else { TEXT_3 })),
                    )
                    .child(section.label())
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.section = section;
                        cx.notify();
                    }))
            }))
    }
}

impl Render for SettingsView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content = match self.section {
            Section::General => self.general.clone().into_any_element(),
            Section::Network => self.network.clone().into_any_element(),
        };
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme::color(BG_0))
            .text_color(theme::color(TEXT_1))
            .text_size(px(13.))
            .child(
                TitleBar::new().child(
                    div()
                        .text_size(px(10.))
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme::color(TEXT_3))
                        .child("SETTINGS"),
                ),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .child(self.render_nav(cx))
                    .child(motion::enter(
                        ("section-in", self.section as usize),
                        div().flex_1().min_w_0().h_full().child(content),
                        true,
                    )),
            )
    }
}

pub fn open(cx: &mut App) {
    if let Some(SettingsWindow(handle)) = cx.try_global::<SettingsWindow>()
        && cx.windows().contains(handle)
    {
        let handle = *handle;
        let _ = handle.update(cx, |_, window, _| window.activate_window());
        return;
    }
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
            None,
            size(px(960.), px(680.)),
            cx,
        ))),
        window_min_size: Some(size(px(760.), px(480.))),
        ..TitleBar::window_options()
    };
    match gpui_kit::open_window(options, cx, |window, cx| {
        window.set_window_title("Telescope Settings");
        cx.new(|cx| SettingsView::new(window, cx))
    }) {
        Ok((handle, _)) => cx.set_global(SettingsWindow(handle)),
        Err(e) => log::error!("Failed to open settings: {}", e),
    }
}
