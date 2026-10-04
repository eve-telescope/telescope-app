// Prevents an additional console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod assets;
mod runtime;
mod services;
mod single_instance;
mod state;
mod theme;
mod ui;
mod views;
mod windows;

use std::sync::Arc;

use gpui_kit::App;
use log::{LevelFilter, info};
use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, TermLogger, TerminalMode, WriteLogger,
};
use telescope_core::paths::AppPaths;

use services::Services;
use single_instance::Startup;
use state::Stores;

fn init_logging(paths: &AppPaths) {
    let config = ConfigBuilder::new()
        .add_filter_allow_str("telescope")
        .build();
    let mut loggers: Vec<Box<dyn simplelog::SharedLogger>> = vec![TermLogger::new(
        LevelFilter::Info,
        config.clone(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )];
    if let Ok(file) = std::fs::File::create(paths.logs.join("telescope.log")) {
        loggers.push(WriteLogger::new(LevelFilter::Info, config, file));
    }
    let _ = CombinedLogger::init(loggers);
}

fn main() {
    // reqwest and GPUI's image client enable different rustls backends, so
    // rustls cannot pick one on its own.
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    let paths = AppPaths::resolve();
    init_logging(&paths);
    info!("Telescope v{} starting", env!("CARGO_PKG_VERSION"));

    let args: Vec<String> = std::env::args().skip(1).collect();
    let forwarded = match single_instance::acquire(&paths.data, &args) {
        Startup::Primary(rx) => rx,
        Startup::Secondary => return,
    };

    let application = gpui_kit::application().with_assets(assets::AppAssets);

    application.on_open_urls(|urls| {
        windows::deep_link::queue(urls);
    });

    application.run(move |cx: &mut App| {
        gpui_kit::init(cx);
        theme::init(cx);

        let user_agent = format!(
            "Telescope/{} (eve-telescope.com; github.com/eve-telescope/telescope-app)",
            env!("CARGO_PKG_VERSION")
        );
        if let Ok(client) = reqwest_client::ReqwestClient::user_agent(&user_agent) {
            cx.set_http_client(Arc::new(client));
        }

        cx.set_global(Services::new(paths));
        Stores::init(cx);

        windows::main_window::open(cx);
        windows::deep_link::init(args, forwarded, cx);
        windows::platform::init(cx);

        cx.on_app_quit(|cx: &mut App| {
            let cache = Services::get(cx).cache.clone();
            async move {
                let _ = cache.flush();
            }
        })
        .detach();
    });
}
