//! Keeps one running instance. A second launch hands its command-line
//! arguments (deep links on Windows and Linux) to the first one and exits.

use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use interprocess::local_socket::{
    GenericNamespaced, ListenerOptions, Stream, ToNsName, traits::ListenerExt, traits::Stream as _,
};
use log::{info, warn};

pub enum Startup {
    /// This is the only instance. Messages from later launches arrive on the
    /// receiver, one per line of their argv (an empty line means "focus").
    Primary(futures::channel::mpsc::UnboundedReceiver<String>),
    Secondary,
}

/// One instance per data directory, so separate profiles don't hand their
/// launches to each other.
fn socket_name(data_dir: &Path) -> String {
    let mut hasher = DefaultHasher::new();
    data_dir.hash(&mut hasher);
    format!("com.timkunze.telescope-{:x}.sock", hasher.finish())
}

pub fn acquire(data_dir: &Path, args: &[String]) -> Startup {
    let socket = socket_name(data_dir);
    let Ok(name) = socket.as_str().to_ns_name::<GenericNamespaced>() else {
        return Startup::Primary(futures::channel::mpsc::unbounded().1);
    };

    if let Ok(mut stream) = Stream::connect(name.clone()) {
        let message = if args.is_empty() {
            "\n".to_string()
        } else {
            args.iter().map(|arg| format!("{arg}\n")).collect()
        };
        if stream.write_all(message.as_bytes()).is_ok() {
            info!("Forwarded launch to the running instance");
            return Startup::Secondary;
        }
    }

    let (tx, rx) = futures::channel::mpsc::unbounded();
    match ListenerOptions::new()
        .name(name)
        .try_overwrite(true)
        .create_sync()
    {
        Ok(listener) => {
            std::thread::Builder::new()
                .name("single-instance".into())
                .spawn(move || {
                    for stream in listener.incoming().filter_map(Result::ok) {
                        for line in BufReader::new(stream).lines().map_while(Result::ok) {
                            let _ = tx.unbounded_send(line);
                        }
                    }
                })
                .ok();
        }
        Err(e) => warn!("Single-instance listener unavailable: {}", e),
    }
    Startup::Primary(rx)
}
