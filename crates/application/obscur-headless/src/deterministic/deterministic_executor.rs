//! A fully deterministic, single-threaded job runner (spec §7.4).

use std::cell::RefCell;
use std::collections::VecDeque;

use obscur_task::BlockingExecutor;

/// A deterministic, single-threaded executor: jobs run in strict FIFO order,
/// exactly when the test drives them — never on a background thread and never
/// reordered.
///
/// Jobs are enqueued by [`spawn`](Self::spawn) (or via the
/// [`BlockingExecutor`] port) and only run when the test calls
/// [`run`](Self::run) or [`run_next`](Self::run_next). Because a job may itself
/// `spawn` more jobs, the internal queue lives behind a [`RefCell`] and jobs are
/// popped out of the borrow before they are invoked ("pop-then-run"), so a job
/// that spawns during its own run never double-borrows the cell.
///
/// This is the Phase 1 model of controlled execution using [`FnOnce`] jobs. A
/// full [`Future`](std::future::Future)-based spawner (polling with a waker) is
/// a later addition; here "blocking" work is simply queued and run inline when
/// the test drives it, so there is never any real blocking or thread.
#[derive(Default)]
pub struct DeterministicExecutor {
    /// Pending jobs, drained front-to-back to guarantee FIFO ordering.
    queue: RefCell<VecDeque<Box<dyn FnOnce()>>>,
}

impl DeterministicExecutor {
    /// Creates an executor with an empty job queue.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enqueues `job` to run later. This does **not** run the job; call
    /// [`run`](Self::run) or [`run_next`](Self::run_next) to drive execution.
    pub fn spawn(&self, job: impl FnOnce() + 'static) {
        self.queue.borrow_mut().push_back(Box::new(job));
    }

    /// Runs queued jobs in FIFO order until the queue is empty.
    ///
    /// A job may itself [`spawn`](Self::spawn) more jobs; those run later within
    /// the same drain, after all jobs already ahead of them, preserving order.
    /// Each job is popped out of the [`RefCell`] borrow before being invoked, so
    /// spawning during a run never conflicts with the borrow.
    pub fn run(&self) {
        while self.run_next() {}
    }

    /// Runs exactly one queued job (the front of the queue).
    ///
    /// Returns `true` if a job was run, or `false` if the queue was empty. The
    /// job is popped while the borrow is held, the borrow is dropped, and only
    /// then is the job invoked — so a job that `spawn`s during its run does not
    /// double-borrow the queue.
    pub fn run_next(&self) -> bool {
        let job = self.queue.borrow_mut().pop_front();
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
        self.queue.borrow().len()
    }

    /// Returns `true` when no jobs are waiting to run.
    #[must_use]
    pub fn is_idle(&self) -> bool {
        self.queue.borrow().is_empty()
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
        self.queue.borrow_mut().push_back(job);
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
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn jobs_run_in_fifo_order() {
        let executor = DeterministicExecutor::new();
        let order = Rc::new(RefCell::new(Vec::<u32>::new()));

        for n in 0..3 {
            let order = Rc::clone(&order);
            executor.spawn(move || order.borrow_mut().push(n));
        }

        assert_eq!(executor.pending(), 3);
        executor.run();
        assert_eq!(*order.borrow(), vec![0, 1, 2]);
        assert!(executor.is_idle());
    }

    #[test]
    fn run_next_runs_exactly_one() {
        let executor = DeterministicExecutor::new();
        let order = Rc::new(RefCell::new(Vec::<u32>::new()));

        for n in 0..2 {
            let order = Rc::clone(&order);
            executor.spawn(move || order.borrow_mut().push(n));
        }

        assert!(executor.run_next());
        assert_eq!(*order.borrow(), vec![0]);
        assert_eq!(executor.pending(), 1);

        assert!(executor.run_next());
        assert_eq!(*order.borrow(), vec![0, 1]);

        assert!(!executor.run_next());
    }

    #[test]
    fn job_spawning_during_run_runs_later_in_same_run() {
        let executor = Rc::new(DeterministicExecutor::new());
        let order = Rc::new(RefCell::new(Vec::<u32>::new()));

        {
            let executor_inner = Rc::clone(&executor);
            let order_first = Rc::clone(&order);
            let order_second = Rc::clone(&order);
            executor.spawn(move || {
                order_first.borrow_mut().push(1);
                // Spawning during a run must not double-borrow the queue.
                executor_inner.spawn(move || order_second.borrow_mut().push(2));
            });
        }

        executor.run();
        assert_eq!(*order.borrow(), vec![1, 2]);
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
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};

        let executor = DeterministicExecutor::new();
        // The port requires `Send` jobs, so capture a `Send`-able flag.
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
}
