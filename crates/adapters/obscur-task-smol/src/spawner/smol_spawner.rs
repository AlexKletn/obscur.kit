//! A smol-backed [`Spawner`] + [`BlockingExecutor`] adapter (spec §2.7 optional
//! task ports — "Опциональные реализации task ports. Ни одна не является core
//! dependency.").
//!
//! [`SmolSpawner`] implements the executor-agnostic [`Spawner`] port on top of
//! [`async_executor::Executor`]. Unlike an ambient runtime, that executor does
//! nothing until a thread drives it, so this adapter owns a worker thread that
//! runs the executor until the spawner is dropped. The kit's core never names
//! smol; this adapter is one concrete way a host wires the port.
//!
//! It also implements [`BlockingExecutor`] (spec §3.3), on smol's *separate*
//! blocking thread pool rather than on that worker — see the impl for why the
//! two must not share a thread. That impl is what lets a `UiRuntime` run its
//! resource loads off the frame (§1.5 "resource loading" as parallel work):
//! before it, the only `BlockingExecutor` in the workspace was the headless
//! `DeterministicExecutor`, which runs every job inline on the caller by
//! design, so nothing could put a real thread behind the port.

use std::fmt;
use std::sync::Arc;
use std::thread::JoinHandle;

use async_executor::Executor;
use obscur_task::{BlockingExecutor, BoxFuture, Spawner};

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

impl BlockingExecutor for SmolSpawner {
    /// Runs `job` on smol's blocking thread pool, off the caller's thread.
    ///
    /// **This does not touch [`Self::executor`], and that is the point.** The
    /// [`Spawner`] half of this type drives its executor with exactly *one*
    /// worker thread, shared with every [`SmolTimer`](crate::timer::SmolTimer)
    /// built from [`SmolSpawner::executor`]. A blocking job — the whole point of
    /// this port is that it blocks — would own that thread for its duration and
    /// stall every future and timer queued behind it. So blocking work goes to
    /// [`blocking::unblock`]'s dedicated, dynamically-grown pool instead, which
    /// is the same split smol itself draws between `smol::spawn` and
    /// `smol::unblock`. The two halves therefore share nothing but the type.
    ///
    /// The returned `Task` is detached: this port is fire-and-forget, and a
    /// detached task still runs — `unblock` schedules its runnable onto the pool
    /// before returning it. A job that panics does not take a pool thread with
    /// it: `blocking` builds its tasks with `propagate_panic(true)`
    /// (`blocking-1.6.2/src/lib.rs:219`), which catches the unwind inside
    /// `Runnable::run` and re-raises it only when the `Task` is polled — and a
    /// detached task is never polled. The result simply never arrives, which for
    /// a resource load means the image is not painted.
    ///
    /// # Panics
    ///
    /// On `wasm` targets `blocking`'s pool cannot exist and `unblock` panics
    /// ("cannot spawn a blocking task on WASM", `blocking-1.6.2/src/lib.rs:212`).
    /// §1.5's WASM workers are a separate, unbuilt facility; do not install this
    /// adapter on a WASM host.
    fn run_blocking(&self, job: Box<dyn FnOnce() + Send + 'static>) {
        blocking::unblock(job).detach();
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

    /// The claim the port's name makes: the job leaves this thread. Asserted on
    /// the `ThreadId`, not on "a spawner exists" — the deterministic executor
    /// also accepts the job and runs it on the *caller's* thread, and only the
    /// id tells the two apart.
    #[test]
    fn run_blocking_runs_the_job_on_another_thread() {
        let spawner = SmolSpawner::new();
        let (tx, rx) = std::sync::mpsc::channel();
        let here = std::thread::current().id();

        BlockingExecutor::run_blocking(
            &spawner,
            Box::new(move || {
                let _ = tx.send(std::thread::current().id());
            }),
        );

        let there = rx
            .recv_timeout(Duration::from_secs(5))
            .expect("the blocking pool ran the job");
        assert_ne!(there, here, "run_blocking must not run on the caller");
    }

    /// The other half of the split the impl's doc claims: blocking work must not
    /// land on the single worker thread `Spawner` drives its executor with, or a
    /// blocking job would stall every future and timer queued behind it. Both
    /// halves are asked for their thread id; three distinct ids is the proof.
    #[test]
    fn blocking_work_does_not_land_on_the_executors_worker_thread() {
        let spawner = SmolSpawner::new();
        let (tx, rx) = std::sync::mpsc::channel();

        let async_tx = tx.clone();
        spawner.spawn(Box::pin(async move {
            let _ = async_tx.send(("spawn", std::thread::current().id()));
        }));
        BlockingExecutor::run_blocking(
            &spawner,
            Box::new(move || {
                let _ = tx.send(("blocking", std::thread::current().id()));
            }),
        );

        let mut seen = std::collections::HashMap::new();
        for _ in 0..2 {
            let (which, id) = rx.recv_timeout(Duration::from_secs(5)).expect("both ran");
            seen.insert(which, id);
        }
        let here = std::thread::current().id();
        assert_ne!(seen["spawn"], here);
        assert_ne!(seen["blocking"], here);
        assert_ne!(
            seen["spawn"], seen["blocking"],
            "blocking work must not occupy the executor's only worker thread"
        );
    }

    #[test]
    fn drop_joins_worker_without_hanging() {
        // If Drop's shutdown were wrong, this join would block forever and hang
        // the suite; reaching the end proves the worker joins cleanly.
        let spawner = SmolSpawner::new();
        drop(spawner);
    }
}
