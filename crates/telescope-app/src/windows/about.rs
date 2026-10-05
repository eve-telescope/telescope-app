use gpui_kit::component::button::Button;
use gpui_kit::component::{Sizable as _, TitleBar};
use gpui_kit::{
    AnyWindowHandle, App, AppContext as _, Bounds, Context, Div, FontWeight, Global, IntoElement,
    ParentElement as _, Render, Styled as _, Window, WindowBounds, WindowOptions, div, img, px,
    size,
};
use telescope_core::view::shortcut::Shortcut;

use crate::state::Stores;
use crate::theme::{self, BG_0, BG_2, BG_3, BORDER, CYAN, TEXT_1, TEXT_2, TEXT_3};
use crate::ui::section_title;

const IS_MAC: bool = cfg!(target_os = "macos");

struct AboutWindow(AnyWindowHandle);

impl Global for AboutWindow {}

struct AboutView;

fn kbd(text: String) -> impl IntoElement {
    div()
        .mt_1()
        .px_2()
        .py_0p5()
        .rounded_sm()
        .bg(theme::color(BG_3))
        .text_xs()
        .font_weight(FontWeight::MEDIUM)
        .child(text)
}

fn step(n: usize, title: &'static str, text: &'static str, keys: String) -> impl IntoElement {
    div()
        .flex()
        .gap_3()
        .child(
            div()
                .size(px(22.))
                .flex_none()
                .rounded_full()
                .bg(theme::tint(CYAN, 0x1a))
                .text_color(theme::color(CYAN))
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .flex()
                .items_center()
                .justify_center()
                .child(n.to_string()),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .items_start()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(title),
                )
                .child(div().text_xs().text_color(theme::color(TEXT_2)).child(text))
                .child(kbd(keys)),
        )
}

fn tip(text: &'static str) -> Div {
    div()
        .flex()
        .gap_2()
        .text_xs()
        .text_color(theme::color(TEXT_2))
        .child(div().text_color(theme::color(CYAN)).child("•"))
        .child(text)
}

fn link(id: &'static str, label: &'static str, url: &'static str) -> impl IntoElement {
    Button::new(id)
        .outline()
        .small()
        .label(label)
        .on_click(move |_, _, cx| cx.open_url(url))
}

impl Render for AboutView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let shortcut = Shortcut::parse(&Stores::get(cx).settings.read(cx).get().global_shortcut)
            .map(|s| s.format(IS_MAC))
            .unwrap_or_default();
        let modifier = if IS_MAC { "⌘" } else { "Ctrl" };

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme::color(BG_0))
            .text_color(theme::color(TEXT_1))
            .child(TitleBar::new())
            .child(
                div()
                    .flex_1()
                    .p_6()
                    .flex()
                    .flex_col()
                    .gap_6()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(img("logo.svg").size(px(64.)))
                            .child(
                                div()
                                    .child(div().text_lg().font_weight(FontWeight::BOLD).child("TELESCOPE"))
                                    .child(div().text_xs().text_color(theme::color(TEXT_2)).child("EVE Online Intel Tool"))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme::color(TEXT_3))
                                            .child(format!("Version {}", env!("CARGO_PKG_VERSION"))),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .child(section_title("HOW TO USE"))
                            .child(step(
                                1,
                                "Select pilots in Local",
                                "In EVE, open Local chat and select all pilots",
                                format!("{modifier} + A"),
                            ))
                            .child(step(2, "Copy to clipboard", "Copy the selected pilot names", format!("{modifier} + C")))
                            .child(step(
                                3,
                                "Scan with Telescope",
                                "Press your global hotkey to paste and scan instantly",
                                shortcut,
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(section_title("TIPS"))
                            .child(tip("Configure a global hotkey to scan directly from EVE without switching windows"))
                            .child(tip("Results are cached, so repeated scans are instant"))
                            .child(tip("Click Share Scan to copy a link for your fleet"))
                            .child(tip("Use the filters on the right to narrow down by corp or alliance")),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(section_title("LINKS"))
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .child(link("link-site", "eve-telescope.com", "https://eve-telescope.com"))
                                    .child(link(
                                        "link-github",
                                        "GitHub",
                                        "https://github.com/eve-telescope/telescope-app",
                                    ))
                                    .child(link("link-zkill", "zKillboard", "https://zkillboard.com")),
                            ),
                    )
                    .child(
                        div()
                            .mt_auto()
                            .pt_4()
                            .border_t_1()
                            .border_color(theme::color(BORDER))
                            .text_size(px(10.))
                            .text_color(theme::color(TEXT_3))
                            .child("Data provided by zKillboard and EVE ESI")
                            .child("EVE Online and all related assets © CCP hf."),
                    ),
            )
            .child(div().h(px(1.)).bg(theme::color(BG_2)))
    }
}

pub fn open(cx: &mut App) {
    if let Some(AboutWindow(handle)) = cx.try_global::<AboutWindow>()
        && cx.windows().contains(handle)
    {
        let handle = *handle;
        let _ = handle.update(cx, |_, window, _| window.activate_window());
        return;
    }
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
            None,
            size(px(500.), px(700.)),
            cx,
        ))),
        is_resizable: false,
        is_minimizable: false,
        ..TitleBar::window_options()
    };
    match gpui_kit::open_window(options, cx, |window, cx| {
        window.set_window_title("About Telescope");
        cx.new(|_| AboutView)
    }) {
        Ok((handle, _)) => cx.set_global(AboutWindow(handle)),
        Err(e) => log::error!("Failed to open about window: {}", e),
    }
}
