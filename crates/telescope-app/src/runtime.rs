//! A tokio runtime for reqwest and the websocket client. GPUI runs its own
//! executors, so tokio futures are spawned here and their join handles are
//! awaited from GPUI tasks.

use std::future::Future;
use std::pin::Pin;
use std::sync::OnceLock;
use std::task::{Context, Poll};

use tokio::runtime::{Handle, Runtime};
use tokio::task::JoinHandle;

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

pub fn handle() -> &'static Handle {
    RUNTIME
        .get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .thread_name("telescope-tokio")
                .enable_all()
                .build()
                .expect("failed to start tokio runtime")
        })
        .handle()
}

/// Aborts the tokio task when dropped, so cancelling the awaiting GPUI task
/// also cancels the work.
pub struct TokioTask<T>(Option<JoinHandle<T>>);

impl<T> TokioTask<T> {
    /// Lets the task run to completion without anyone awaiting it.
    pub fn detach(mut self) {
        self.0.take();
    }
}

impl<T> Drop for TokioTask<T> {
    fn drop(&mut self) {
        if let Some(handle) = &self.0 {
            handle.abort();
        }
    }
}

impl<T> Future for TokioTask<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        let handle = self.0.as_mut().expect("polled after detach");
        Pin::new(handle)
            .poll(cx)
            .map(|result| result.expect("tokio task panicked"))
    }
}

pub fn spawn<F>(future: F) -> TokioTask<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    TokioTask(Some(handle().spawn(future)))
}
