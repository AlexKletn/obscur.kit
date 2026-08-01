#![doc = include_str!("smol_spawner.doc/_module.md")]

use std::fmt;
use std::sync::Arc;
use std::thread::JoinHandle;

use async_executor::Executor;
use obscur_task::{BlockingExecutor, BoxFuture, Spawner};

#[doc = include_str!("smol_spawner.doc/SmolSpawner.md")]
pub struct SmolSpawner {
    #[doc = include_str!("smol_spawner.doc/executor.md")]
    executor: Arc<Executor<'static>>,
    #[doc = include_str!("smol_spawner.doc/shutdown.md")]
    shutdown: async_channel::Sender<()>,
    #[doc = include_str!("smol_spawner.doc/worker.md")]
    worker: Option<JoinHandle<()>>,
}

impl SmolSpawner {
    #[doc = include_str!("smol_spawner.doc/new.md")]
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

    #[doc = include_str!("smol_spawner.doc/executor_2.md")]
    #[must_use]
    pub fn executor(&self) -> Arc<Executor<'static>> {
        Arc::clone(&self.executor)
    }
}

impl Default for SmolSpawner {
    #[doc = include_str!("smol_spawner.doc/default.md")]
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for SmolSpawner {
    #[doc = include_str!("smol_spawner.doc/fmt.md")]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SmolSpawner").finish_non_exhaustive()
    }
}

impl Spawner for SmolSpawner {
    #[doc = include_str!("smol_spawner.doc/spawn.md")]
    fn spawn(&self, future: BoxFuture) {
        self.executor.spawn(future).detach();
    }
}

impl BlockingExecutor for SmolSpawner {
    #[doc = include_str!("smol_spawner.doc/run_blocking.md")]
    fn run_blocking(&self, job: Box<dyn FnOnce() + Send + 'static>) {
        blocking::unblock(job).detach();
    }
}

impl Drop for SmolSpawner {
    #[doc = include_str!("smol_spawner.doc/drop.md")]
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
