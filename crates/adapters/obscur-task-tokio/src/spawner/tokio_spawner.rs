//! A Tokio-backed [`Spawner`] adapter (spec §2.7 optional task ports —
//! "Опциональные реализации task ports. Ни одна не является core dependency.").
//!
//! [`TokioSpawner`] implements the executor-agnostic [`Spawner`] port by
//! forwarding each [`BoxFuture`] to a Tokio [`Handle`], the runtime the host
//! chose to bring into its reach. The kit's core never names Tokio; this adapter
//! is one concrete way a host wires the port.

use obscur_task::{BoxFuture, Spawner};
use tokio::runtime::Handle;

/// A [`Spawner`] that schedules `Send` futures onto a Tokio runtime through a
/// [`Handle`] (spec §2.7 optional task ports).
///
/// The handle is cheap to clone and refers to a runtime the host owns; this
/// adapter merely forwards work to it and never drives the runtime itself.
#[derive(Debug, Clone)]
pub struct TokioSpawner {
    /// The Tokio runtime handle work is spawned onto.
    handle: Handle,
}

impl TokioSpawner {
    /// Creates a spawner that schedules onto the given Tokio [`Handle`].
    #[must_use]
    pub fn new(handle: Handle) -> Self {
        Self { handle }
    }

    /// Creates a spawner bound to the Tokio runtime of the current context.
    ///
    /// # Panics
    ///
    /// Panics if called outside a Tokio runtime context, matching the standard
    /// behavior of [`Handle::current`].
    #[must_use]
    pub fn current() -> Self {
        Self {
            handle: Handle::current(),
        }
    }

    /// Returns the Tokio [`Handle`] this spawner schedules onto.
    #[must_use]
    pub fn handle(&self) -> &Handle {
        &self.handle
    }
}

impl Spawner for TokioSpawner {
    /// Spawns `future` onto the Tokio runtime.
    ///
    /// [`Handle::spawn`] accepts any `Future<Output = ()> + Send + 'static`, which
    /// [`BoxFuture`] satisfies. The returned `JoinHandle` is dropped: the port is
    /// fire-and-forget, so the task runs detached and its completion is not
    /// awaited here.
    fn spawn(&self, future: BoxFuture) {
        self.handle.spawn(future);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;

    #[test]
    fn spawned_future_runs() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .expect("build current-thread runtime");

        rt.block_on(async {
            let spawner = TokioSpawner::current();
            let flag = Arc::new(AtomicBool::new(false));
            let flag_in_task = Arc::clone(&flag);

            let future: BoxFuture = Box::pin(async move {
                flag_in_task.store(true, Ordering::SeqCst);
            });
            spawner.spawn(future);

            // Yield so the current-thread runtime can drive the spawned task,
            // then sleep with a generous margin as a robust fallback.
            tokio::task::yield_now().await;
            tokio::time::sleep(Duration::from_millis(20)).await;

            assert!(flag.load(Ordering::SeqCst));
        });
    }
}
