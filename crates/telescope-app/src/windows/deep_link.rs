//! telescope:// links. macOS delivers them through `on_open_urls`; Windows and
//! Linux pass them as arguments, either to this launch or forwarded from a
//! second launch by the single-instance socket.

use std::sync::OnceLock;

use futures::StreamExt as _;
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use gpui_kit::{App, AsyncApp};
use log::{error, info, warn};
use telescope_core::domain::deeplink::{DeepLink, parse};
use telescope_core::share::fetch_share;

use crate::runtime;
use crate::services::Services;
use crate::state::Stores;
use crate::windows::main_window;

type Queue = (
    UnboundedSender<String>,
    std::sync::Mutex<Option<UnboundedReceiver<String>>>,
);

static QUEUE: OnceLock<Queue> = OnceLock::new();

fn queue_channel() -> &'static Queue {
    QUEUE.get_or_init(|| {
        let (tx, rx) = unbounded();
        (tx, std::sync::Mutex::new(Some(rx)))
    })
}

/// Called from the platform's open-URL callback, possibly before the app has
/// finished launching.
pub fn queue(urls: Vec<String>) {
    for url in urls {
        let _ = queue_channel().0.unbounded_send(url);
    }
}

pub fn init(args: Vec<String>, forwarded: UnboundedReceiver<String>, cx: &mut App) {
    queue(
        args.into_iter()
            .filter(|arg| arg.starts_with("telescope:"))
            .collect(),
    );
    let urls = queue_channel().1.lock().ok().and_then(|mut rx| rx.take());

    if let Some(mut urls) = urls {
        cx.spawn(async move |cx: &mut AsyncApp| {
            while let Some(url) = urls.next().await {
                cx.update(|cx| handle(&url, cx));
            }
        })
        .detach();
    }

    let mut forwarded = forwarded;
    cx.spawn(async move |cx: &mut AsyncApp| {
        while let Some(line) = forwarded.next().await {
            cx.update(|cx| {
                main_window::focus(cx);
                if line.starts_with("telescope:") {
                    handle(&line, cx);
                }
            });
        }
    })
    .detach();
}

fn handle(url: &str, cx: &mut App) {
    info!("[DeepLink] Received {}", url);
    let stores = Stores::get(cx);
    match parse(url) {
        Some(DeepLink::Auth { token }) => {
            stores
                .intel
                .update(cx, |intel, cx| intel.set_api_token(token, cx));
        }
        Some(DeepLink::Share { code }) => {
            let base_url = Services::get(cx).api_base_url.clone();
            let load = runtime::spawn(async move { fetch_share(&base_url, &code).await });
            cx.spawn(async move |cx: &mut AsyncApp| match load.await {
                Ok(share) => {
                    info!("[DeepLink] Loaded {} pilots from share", share.pilots.len());
                    let text = share.pilots.join("\n");
                    cx.update(|cx| {
                        main_window::focus(cx);
                        main_window::scan(&text, cx);
                    });
                }
                Err(e) => error!("[DeepLink] {}", e),
            })
            .detach();
        }
        None => warn!("[DeepLink] Ignoring unrecognized URL: {}", url),
    }
}
