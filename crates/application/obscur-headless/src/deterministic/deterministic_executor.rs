//! A fully deterministic, single-threaded job runner (spec §7.4).

use std::collections::VecDeque;
use std::sync::Mutex;

use obscur_task::BlockingExecutor;

/// A deterministic, single-threaded executor: jobs run in strict FIFO order,
/// exactly when the test drives them — never on a background thread and never
/// reordered.
///
/// Jobs are enqueued by [`spawn`](Self::spawn) (or via the
/// [`BlockingExecutor`] port) and only run when the test calls
/// [`run`](Self::run) or [`run_next`](Self::run_next). Because a job may itself
/// `spawn` more jobs, the internal queue lives behind a [`Mutex`] and jobs are
/// popped out of the guard before they are invoked ("pop-then-run"), so a job
/// that spawns during its own run never deadlocks on the lock it already holds.
///
/// The queue is a [`Mutex`] rather than a `RefCell`, and jobs are required to be
/// [`Send`], purely so this type satisfies the `BlockingExecutor + Send + Sync`
/// bound on
/// [`UiRuntime::set_blocking_executor`](obscur_runtime::UiRuntime::set_blocking_executor).
/// That bound is what a real thread-pool adapter needs; without it the *only*
/// executor a headless test could install would be a hand-rolled double, and the
/// deterministic one — the one §7.4 actually specifies — could not be wired to a
/// runtime at all. Nothing here ever touches a second thread.
///
/// This is the Phase 1 model of controlled execution using [`FnOnce`] jobs. A
/// full [`Future`](std::future::Future)-based spawner (polling with a waker) is
/// a later addition; here "blocking" work is simply queued and run inline when
/// the test drives it, so there is never any real blocking or thread.
#[derive(Default)]
pub struct DeterministicExecutor {
    /// Pending jobs, drained front-to-back to guarantee FIFO ordering.
    queue: Mutex<VecDeque<Box<dyn FnOnce() + Send>>>,
}

impl DeterministicExecutor {
    /// Creates an executor with an empty job queue.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enqueues `job` to run later. This does **not** run the job; call
    /// [`run`](Self::run) or [`run_next`](Self::run_next) to drive execution.
    ///
    /// `job` must be [`Send`] because the same queue backs the
    /// [`BlockingExecutor`] port, whose jobs are `Send`. Nothing is ever sent
    /// anywhere: the job runs on the calling thread, when the test says so.
    pub fn spawn(&self, job: impl FnOnce() + Send + 'static) {
        self.enqueue(Box::new(job));
    }

    /// Pushes an already-boxed job onto the tail of the queue.
    fn enqueue(&self, job: Box<dyn FnOnce() + Send>) {
        self.lock().push_back(job);
    }

    /// Locks the queue, recovering from a poisoned lock.
    ///
    /// A job that panics while the queue is *unlocked* cannot poison it, but a
    /// panic elsewhere holding the guard would; recovering keeps a test's failure
    /// message the original panic rather than a confusing poison error.
    fn lock(&self) -> std::sync::MutexGuard<'_, VecDeque<Box<dyn FnOnce() + Send>>> {
        self.queue
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Runs queued jobs in FIFO order until the queue is empty.
    ///
    /// A job may itself [`spawn`](Self::spawn) more jobs; those run later within
    /// the same drain, after all jobs already ahead of them, preserving order.
    /// Each job is popped out of the lock before being invoked, so spawning
    /// during a run never contends with the guard.
    pub fn run(&self) {
        while self.run_next() {}
    }

    /// Runs exactly one queued job (the front of the queue).
    ///
    /// Returns `true` if a job was run, or `false` if the queue was empty. The
    /// job is popped while the lock is held, the guard is dropped, and only
    /// then is the job invoked — so a job that `spawn`s during its run does not
    /// deadlock on the queue.
    pub fn run_next(&self) -> bool {
        let job = self.lock().pop_front();
        match job {
            Some(job) => {
                job();
                true
            }
            None => false,
        }
    }

    /// Returns the number of jobs waiting to run.
    #[must_use]
    pub fn pending(&self) -> usize {
        self.lock().len()
    }

    /// Returns `true` when no jobs are waiting to run.
    #[must_use]
    pub fn is_idle(&self) -> bool {
        self.lock().is_empty()
    }
}

impl BlockingExecutor for DeterministicExecutor {
    /// Enqueues the (already-boxed) blocking `job`.
    ///
    /// This executor is deterministic and single-threaded: "blocking" work is
    /// just queued and run inline when the test drives execution via
    /// [`run`](Self::run) / [`run_next`](Self::run_next). There is never any
    /// real blocking or a background thread.
    fn run_blocking(&self, job: Box<dyn FnOnce() + Send + 'static>) {
        self.enqueue(job);
    }
}

impl std::fmt::Debug for DeterministicExecutor {
    /// Reports the pending job count; the boxed closures themselves are opaque.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeterministicExecutor")
            .field("pending", &self.pending())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// A `Send` order log: the port's jobs must be `Send`, so the shared state a
    /// test job captures must be too.
    fn log() -> Arc<Mutex<Vec<u32>>> {
        Arc::new(Mutex::new(Vec::new()))
    }

    fn entries(log: &Arc<Mutex<Vec<u32>>>) -> Vec<u32> {
        log.lock().expect("lock").clone()
    }

    #[test]
    fn jobs_run_in_fifo_order() {
        let executor = DeterministicExecutor::new();
        let order = log();

        for n in 0..3 {
            let order = Arc::clone(&order);
            executor.spawn(move || order.lock().expect("lock").push(n));
        }

        assert_eq!(executor.pending(), 3);
        executor.run();
        assert_eq!(entries(&order), vec![0, 1, 2]);
        assert!(executor.is_idle());
    }

    #[test]
    fn run_next_runs_exactly_one() {
        let executor = DeterministicExecutor::new();
        let order = log();

        for n in 0..2 {
            let order = Arc::clone(&order);
            executor.spawn(move || order.lock().expect("lock").push(n));
        }

        assert!(executor.run_next());
        assert_eq!(entries(&order), vec![0]);
        assert_eq!(executor.pending(), 1);

        assert!(executor.run_next());
        assert_eq!(entries(&order), vec![0, 1]);

        assert!(!executor.run_next());
    }

    #[test]
    fn job_spawning_during_run_runs_later_in_same_run() {
        let executor = Arc::new(DeterministicExecutor::new());
        let order = log();

        {
            let executor_inner = Arc::clone(&executor);
            let order_first = Arc::clone(&order);
            let order_second = Arc::clone(&order);
            executor.spawn(move || {
                order_first.lock().expect("lock").push(1);
                // Spawning during a run must not deadlock on the queue lock.
                executor_inner.spawn(move || order_second.lock().expect("lock").push(2));
            });
        }

        executor.run();
        assert_eq!(entries(&order), vec![1, 2]);
        assert!(executor.is_idle());
    }

    #[test]
    fn is_idle_and_pending_reflect_queue() {
        let executor = DeterministicExecutor::new();
        assert!(executor.is_idle());
        assert_eq!(executor.pending(), 0);

        executor.spawn(|| {});
        assert!(!executor.is_idle());
        assert_eq!(executor.pending(), 1);

        executor.run();
        assert!(executor.is_idle());
    }

    #[test]
    fn run_blocking_enqueues_and_run_executes_it() {
        use std::sync::atomic::{AtomicBool, Ordering};

        let executor = DeterministicExecutor::new();
        let ran = Arc::new(AtomicBool::new(false));

        {
            let ran = Arc::clone(&ran);
            let job: Box<dyn FnOnce() + Send + 'static> =
                Box::new(move || ran.store(true, Ordering::SeqCst));
            executor.run_blocking(job);
        }

        assert_eq!(executor.pending(), 1);
        assert!(!ran.load(Ordering::SeqCst));
        executor.run();
        assert!(ran.load(Ordering::SeqCst));
    }

    /// The bound that lets a headless host install this on a `UiRuntime`
    /// (`set_blocking_executor` takes `Arc<dyn BlockingExecutor + Send + Sync>`).
    /// Without it the deterministic executor cannot reach a real frame at all.
    #[test]
    fn is_send_and_sync_so_a_runtime_can_hold_it() {
        fn assert_port(_: Arc<dyn BlockingExecutor + Send + Sync>) {}
        assert_port(Arc::new(DeterministicExecutor::new()));
    }
}
