//! A smol-backed [`Timer`] adapter (spec §2.7 optional task ports —
//! "Опциональные реализации task ports. Ни одна не является core dependency.";
//! §3.5 timer facility).
//!
//! [`Timer::schedule`] carries only a delay, no callback, so the wake action is
//! supplied once at construction. [`SmolTimer`] schedules a detached task onto a
//! shared [`async_executor::Executor`] that sleeps for the requested delay via
//! [`async_io::Timer::after`] and then invokes that wake action; the domain
//! wires the action to request a frame (spec §3.5).

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use async_executor::Executor;
use obscur_task::Timer;

/// A fire-and-forget wake [`Timer`] backed by smol (spec §2.7 optional task
/// ports, §3.5 timer facility).
///
/// Each [`SmolTimer::schedule`] spawns a detached task onto the shared executor
/// that awaits an [`async_io::Timer`] for the delay and then calls the stored
/// wake action. The executor must be the one a
/// [`SmolSpawner`](crate::spawner::SmolSpawner) drives — obtained from
/// [`SmolSpawner::executor`](crate::spawner::SmolSpawner::executor) — so the
/// scheduled task actually runs. Because the port has no callback parameter, the
/// action is fixed at construction. The timer performs no coalescing or
/// de-duplication: every `schedule` arms an independent wake, and collapsing
/// redundant wakes is the scheduler's responsibility.
#[derive(Clone)]
pub struct SmolTimer {
    /// The shared executor wake tasks are spawned onto. Must be a driven
    /// executor (typically a `SmolSpawner`'s) for scheduled wakes to fire.
    executor: Arc<Executor<'static>>,
    /// The action invoked after each scheduled delay elapses. Shared so the
    /// timer stays [`Clone`] and each spawned task can hold its own reference.
    wake: Arc<dyn Fn() + Send + Sync + 'static>,
}

impl SmolTimer {
    /// Creates a timer that, for each scheduled delay, wakes by calling `wake`
    /// on a task spawned onto `executor`.
    ///
    /// Pass the executor of the [`SmolSpawner`](crate::spawner::SmolSpawner)
    /// driving this timer's tasks so scheduled wakes actually run.
    #[must_use]
    pub fn new(executor: Arc<Executor<'static>>, wake: impl Fn() + Send + Sync + 'static) -> Self {
        Self {
            executor,
            wake: Arc::new(wake),
        }
    }
}

impl fmt::Debug for SmolTimer {
    /// Formats without the wake closure, which is not itself `Debug`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SmolTimer").finish_non_exhaustive()
    }
}

impl Timer for SmolTimer {
    /// Schedules a wake-up after `delay`.
    ///
    /// This spawns a detached task on the shared executor that awaits an
    /// [`async_io::Timer::after`] for `delay` and then invokes the wake action.
    /// It is fire-and-forget: the task is not tracked, nothing is awaited, and
    /// coalescing or de-duplicating overlapping wakes is the scheduler's job,
    /// not this adapter's.
    fn schedule(&self, delay: Duration) {
        let wake = Arc::clone(&self.wake);
        self.executor
            .spawn(async move {
                async_io::Timer::after(delay).await;
                wake();
            })
            .detach();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spawner::SmolSpawner;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn schedule_wakes_after_delay() {
        let spawner = SmolSpawner::new();
        let flag = Arc::new(AtomicBool::new(false));
        let flag_for_wake = Arc::clone(&flag);

        let timer = SmolTimer::new(spawner.executor(), move || {
            flag_for_wake.store(true, Ordering::SeqCst);
        });
        timer.schedule(Duration::from_millis(5));

        // Bounded poll over the 5ms delay; wait up to ~500ms without asserting
        // on exact timing.
        for _ in 0..100 {
            if flag.load(Ordering::SeqCst) {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }

        assert!(flag.load(Ordering::SeqCst));
    }
}
