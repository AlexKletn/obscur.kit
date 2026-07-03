//! A smol-backed [`Spawner`] adapter (spec §2.7 optional task ports —
//! "Опциональные реализации task ports. Ни одна не является core dependency.").
//!
//! [`SmolSpawner`] implements the executor-agnostic [`Spawner`] port on top of
//! [`async_executor::Executor`]. Unlike an ambient runtime, that executor does
//! nothing until a thread drives it, so this adapter owns a worker thread that
//! runs the executor until the spawner is dropped. The kit's core never names
//! smol; this adapter is one concrete way a host wires the port.

use std::fmt;
use std::sync::Arc;
use std::thread::JoinHandle;

use async_executor::Executor;
use obscur_task::{BoxFuture, Spawner};

/// A [`Spawner`] that schedules `Send` futures onto a smol
/// [`async_executor::Executor`] driven by an owned worker thread (spec §2.7
/// optional task ports).
///
/// The executor is shared behind an [`Arc`]; the worker thread calls
/// [`Executor::run`] to drive every spawned task until shutdown. Because the
/// executor is shared, a [`SmolTimer`](crate::timer::SmolTimer) built from
/// [`SmolSpawner::executor`] schedules its wake tasks onto the same running
/// executor, which is required for those timers to fire.
pub struct SmolSpawner {
    /// The shared executor spawned work runs on; also handed to timers so they
    /// schedule onto the same driven executor.
    executor: Arc<Executor<'static>>,
    /// Shutdown signal: sending (or dropping) this completes the worker's
    /// [`Executor::run`] future so the thread returns cleanly.
    shutdown: async_channel::Sender<()>,
    /// The worker thread driving the executor. `Option` so [`Drop`] can take and
    /// join it exactly once.
    worker: Option<JoinHandle<()>>,
}

impl SmolSpawner {
    /// Creates a spawner with a fresh executor and a worker thread that drives
    /// it until the spawner is dropped.
    ///
    /// # Panics
    ///
    /// Panics if the worker thread cannot be spawned by the OS.
    #[must_use]
    pub fn new() -> Self {
        let executor = Arc::new(Executor::new());
        let (tx, rx) = async_channel::bounded::<()>(1);

        let exec = Arc::clone(&executor);
        let worker = std::thread::Builder::new()
            .name("obscur-smol".into())
            .spawn(move || {
                // `run` drives all tasks spawned onto the executor until the
                // supplied future resolves; the shutdown recv resolves once the
                // sender is signalled or dropped.
                futures_lite::future::block_on(exec.run(async move {
                    let _ = rx.recv().await;
                }));
            })
            .expect("spawn worker");

        Self {
            executor,
            shutdown: tx,
            worker: Some(worker),
        }
    }

    /// Returns a handle to the shared executor.
    ///
    /// Cloning the [`Arc`] lets a [`SmolTimer`](crate::timer::SmolTimer) schedule
    /// wake tasks onto the same executor this spawner drives, which is required
    /// for those timers to run.
    #[must_use]
    pub fn executor(&self) -> Arc<Executor<'static>> {
        Arc::clone(&self.executor)
    }
}

impl Default for SmolSpawner {
    /// Equivalent to [`SmolSpawner::new`].
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for SmolSpawner {
    /// Formats without the executor internals, shutdown channel, or worker
    /// handle, none of which carry a useful `Debug` representation here.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SmolSpawner").finish_non_exhaustive()
    }
}

impl Spawner for SmolSpawner {
    /// Spawns `future` onto the executor.
    ///
    /// [`Executor::spawn`] returns a `Task` handle; it is immediately
    /// [`detach`](async_executor::Task::detach)ed so the work runs
    /// fire-and-forget on the worker thread and its completion is not awaited
    /// here.
    fn spawn(&self, future: BoxFuture) {
        self.executor.spawn(future).detach();
    }
}

impl Drop for SmolSpawner {
    /// Signals shutdown so the worker's [`Executor::run`] future resolves, then
    /// joins the worker thread so no thread is leaked.
    ///
    /// Sending on the bounded channel (or its later drop) completes the recv the
    /// worker awaits, which ends `run` and lets the thread finish; the join then
    /// returns promptly rather than blocking.
    fn drop(&mut self) {
        let _ = self.shutdown.try_send(());
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;

    #[test]
    fn spawned_future_runs() {
        let spawner = SmolSpawner::new();
        let flag = Arc::new(AtomicBool::new(false));
        let flag_in_task = Arc::clone(&flag);

        let future: BoxFuture = Box::pin(async move {
            flag_in_task.store(true, Ordering::SeqCst);
        });
        spawner.spawn(future);

        // Bounded poll: the worker thread drives the task; wait up to ~500ms
        // for it to complete without asserting on exact timing.
        for _ in 0..100 {
            if flag.load(Ordering::SeqCst) {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }

        assert!(flag.load(Ordering::SeqCst));
    }

    #[test]
    fn drop_joins_worker_without_hanging() {
        // If Drop's shutdown were wrong, this join would block forever and hang
        // the suite; reaching the end proves the worker joins cleanly.
        let spawner = SmolSpawner::new();
        drop(spawner);
    }
}
