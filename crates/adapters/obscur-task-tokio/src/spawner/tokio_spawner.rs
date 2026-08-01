#![doc = include_str!("tokio_spawner.doc/_module.md")]

use obscur_task::{BoxFuture, Spawner};
use tokio::runtime::Handle;

#[doc = include_str!("tokio_spawner.doc/TokioSpawner.md")]
#[derive(Debug, Clone)]
pub struct TokioSpawner {
    #[doc = include_str!("tokio_spawner.doc/handle.md")]
    handle: Handle,
}

impl TokioSpawner {
    #[doc = include_str!("tokio_spawner.doc/new.md")]
    #[must_use]
    pub fn new(handle: Handle) -> Self {
        Self { handle }
    }

    #[doc = include_str!("tokio_spawner.doc/current.md")]
    #[must_use]
    pub fn current() -> Self {
        Self {
            handle: Handle::current(),
        }
    }

    #[doc = include_str!("tokio_spawner.doc/handle_2.md")]
    #[must_use]
    pub fn handle(&self) -> &Handle {
        &self.handle
    }
}

impl Spawner for TokioSpawner {
    #[doc = include_str!("tokio_spawner.doc/spawn.md")]
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
