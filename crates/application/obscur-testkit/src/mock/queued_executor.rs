//! The blocking port that runs nothing until told.

use std::sync::Mutex;

use obscur_task::BlockingExecutor;

#[doc = include_str!("../../docs/api/mock/queued_executor.doc/QueuedExecutor.md")]
#[derive(Default)]
pub struct QueuedExecutor {
    /// The jobs handed over and not yet run, oldest first.
    jobs: Mutex<Vec<Box<dyn FnOnce() + Send + 'static>>>,
}

impl QueuedExecutor {
    #[doc = include_str!("../../docs/api/mock/queued_executor.doc/run_all.md")]
    pub fn run_all(&self) {
        let queued: Vec<_> = std::mem::take(&mut *self.jobs.lock().expect("lock"));
        for job in queued {
            job();
        }
    }

    #[doc = include_str!("../../docs/api/mock/queued_executor.doc/pending.md")]
    #[must_use]
    pub fn pending(&self) -> usize {
        self.jobs.lock().expect("lock").len()
    }
}

impl BlockingExecutor for QueuedExecutor {
    fn run_blocking(&self, job: Box<dyn FnOnce() + Send + 'static>) {
        self.jobs.lock().expect("lock").push(job);
    }
}

impl std::fmt::Debug for QueuedExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueuedExecutor")
            .field("pending", &self.pending())
            .finish()
    }
}
