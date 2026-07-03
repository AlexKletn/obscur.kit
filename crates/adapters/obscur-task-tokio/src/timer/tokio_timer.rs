//! A Tokio-backed [`Timer`] adapter (spec §2.7 optional task ports —
//! "Опциональные реализации task ports. Ни одна не является core dependency.";
//! §3.5 timer facility).
//!
//! [`Timer::schedule`] carries only a delay, no callback, so the wake action is
//! supplied once at construction. [`TokioTimer`] schedules a detached Tokio task
//! that sleeps for the requested delay and then invokes that wake action; the
//! domain wires the action to request a frame (spec §3.5).

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use obscur_task::Timer;
use tokio::runtime::Handle;

/// A fire-and-forget wake [`Timer`] backed by Tokio (spec §2.7 optional task
/// ports, §3.5 timer facility).
///
/// Each [`TokioTimer::schedule`] spawns a detached task on the Tokio [`Handle`]
/// that sleeps for the delay and then calls the stored wake action. Because the
/// port has no callback parameter, the action is fixed at construction. The timer
/// performs no coalescing or de-duplication: every `schedule` arms an independent
/// wake, and collapsing redundant wakes is the scheduler's responsibility.
#[derive(Clone)]
pub struct TokioTimer {
    /// The Tokio runtime handle wake tasks are spawned onto.
    handle: Handle,
    /// The action invoked after each scheduled delay elapses. Shared so the
    /// timer stays [`Clone`] and each spawned task can hold its own reference.
    wake: Arc<dyn Fn() + Send + Sync + 'static>,
}

impl TokioTimer {
    /// Creates a timer that, for each scheduled delay, wakes by calling `wake` on
    /// the Tokio runtime behind `handle`.
    #[must_use]
    pub fn new(handle: Handle, wake: impl Fn() + Send + Sync + 'static) -> Self {
        Self {
            handle,
            wake: Arc::new(wake),
        }
    }

    /// Returns the Tokio [`Handle`] wake tasks are spawned onto.
    #[must_use]
    pub fn handle(&self) -> &Handle {
        &self.handle
    }
}

impl fmt::Debug for TokioTimer {
    /// Formats without the wake closure, which is not itself `Debug`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TokioTimer").finish_non_exhaustive()
    }
}

impl Timer for TokioTimer {
    /// Schedules a wake-up after `delay`.
    ///
    /// This spawns a detached Tokio task that sleeps for `delay` and then invokes
    /// the wake action. It is fire-and-forget: the task is not tracked, nothing is
    /// awaited, and coalescing or de-duplicating overlapping wakes is the
    /// scheduler's job, not this adapter's.
    fn schedule(&self, delay: Duration) {
        let wake = Arc::clone(&self.wake);
        self.handle.spawn(async move {
            tokio::time::sleep(delay).await;
            wake();
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn schedule_wakes_after_delay() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .expect("build current-thread runtime");

        rt.block_on(async {
            let flag = Arc::new(AtomicBool::new(false));
            let flag_for_wake = Arc::clone(&flag);

            let timer = TokioTimer::new(Handle::current(), move || {
                flag_for_wake.store(true, Ordering::SeqCst);
            });
            timer.schedule(Duration::from_millis(5));

            // Generous margin over the 5ms delay; no exact-timing assertion.
            tokio::time::sleep(Duration::from_millis(30)).await;

            assert!(flag.load(Ordering::SeqCst));
        });
    }
}
