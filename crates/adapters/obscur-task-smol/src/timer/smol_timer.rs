#![doc = include_str!("../../docs/api/timer/smol_timer.doc/_module.md")]

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use async_executor::Executor;
use obscur_task::Timer;

#[doc = include_str!("../../docs/api/timer/smol_timer.doc/SmolTimer.md")]
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
    #[doc = include_str!("../../docs/api/timer/smol_timer.doc/new.md")]
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
    #[doc = include_str!("../../docs/api/timer/smol_timer.doc/schedule.md")]
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
