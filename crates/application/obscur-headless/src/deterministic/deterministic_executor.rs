//! A fully deterministic, single-threaded job runner (spec §7.4).

use std::sync::Mutex;

use obscur_task::BlockingExecutor;

use super::job_queue::JobQueue;

#[doc = include_str!("../../docs/api/deterministic/deterministic_executor.doc/DeterministicExecutor.md")]
#[derive(Default)]
pub struct DeterministicExecutor {
    /// Pending jobs, drained front-to-back to guarantee FIFO ordering.
    queue: Mutex<JobQueue>,
}

impl DeterministicExecutor {
    /// Creates an executor with an empty job queue.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("../../docs/api/deterministic/deterministic_executor.doc/spawn.md")]
    pub fn spawn(&self, job: impl FnOnce() + Send + 'static) {
        self.enqueue(Box::new(job));
    }

    /// Pushes an already-boxed job onto the tail of the queue.
    fn enqueue(&self, job: Box<dyn FnOnce() + Send>) {
        let mut queue = self.lock();
        let id = queue.next_id;
        queue.next_id += 1;
        queue.jobs.push_back((id, job));
    }

    #[doc = include_str!("../../docs/api/deterministic/deterministic_executor.doc/lock.md")]
    fn lock(&self) -> std::sync::MutexGuard<'_, JobQueue> {
        self.queue
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    #[doc = include_str!("../../docs/api/deterministic/deterministic_executor.doc/run.md")]
    pub fn run(&self) {
        while self.run_next() {}
    }

    #[doc = include_str!("../../docs/api/deterministic/deterministic_executor.doc/run_next.md")]
    pub fn run_next(&self) -> bool {
        let job = self.lock().jobs.pop_front();
        match job {
            Some((_, job)) => {
                job();
                true
            }
            None => false,
        }
    }

    /// Lists queued operation IDs in submission order.
    #[must_use]
    pub fn pending_operations(&self) -> Vec<u64> {
        self.lock().jobs.iter().map(|(id, _)| *id).collect()
    }

    /// Runs one pending operation by ID.
    pub fn run_operation(&self, operation: u64) -> bool {
        let job = {
            let mut queue = self.lock();
            queue
                .jobs
                .iter()
                .position(|(id, _)| *id == operation)
                .and_then(|index| queue.jobs.remove(index))
        };
        if let Some((_, job)) = job {
            job();
            true
        } else {
            false
        }
    }

    /// Returns the number of jobs waiting to run.
    #[must_use]
    pub fn pending(&self) -> usize {
        self.lock().jobs.len()
    }

    /// Returns `true` when no jobs are waiting to run.
    #[must_use]
    pub fn is_idle(&self) -> bool {
        self.lock().jobs.is_empty()
    }
}

impl BlockingExecutor for DeterministicExecutor {
    #[doc = include_str!("../../docs/api/deterministic/deterministic_executor.doc/run_blocking.md")]
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
