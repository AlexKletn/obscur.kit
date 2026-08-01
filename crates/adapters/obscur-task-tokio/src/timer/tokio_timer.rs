#![doc = include_str!("tokio_timer.doc/_module.md")]

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use obscur_task::Timer;
use tokio::runtime::Handle;

#[doc = include_str!("tokio_timer.doc/TokioTimer.md")]
#[derive(Clone)]
pub struct TokioTimer {
    #[doc = include_str!("tokio_timer.doc/handle.md")]
    handle: Handle,
    #[doc = include_str!("tokio_timer.doc/wake.md")]
    wake: Arc<dyn Fn() + Send + Sync + 'static>,
}

impl TokioTimer {
    #[doc = include_str!("tokio_timer.doc/new.md")]
    #[must_use]
    pub fn new(handle: Handle, wake: impl Fn() + Send + Sync + 'static) -> Self {
        Self {
            handle,
            wake: Arc::new(wake),
        }
    }

    #[doc = include_str!("tokio_timer.doc/handle_2.md")]
    #[must_use]
    pub fn handle(&self) -> &Handle {
        &self.handle
    }
}

impl fmt::Debug for TokioTimer {
    #[doc = include_str!("tokio_timer.doc/fmt.md")]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TokioTimer").finish_non_exhaustive()
    }
}

impl Timer for TokioTimer {
    #[doc = include_str!("tokio_timer.doc/schedule.md")]
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
