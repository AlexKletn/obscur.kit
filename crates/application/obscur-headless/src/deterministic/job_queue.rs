use std::collections::VecDeque;

type Job = Box<dyn FnOnce() + Send>;

#[derive(Default)]
pub(super) struct JobQueue {
    pub(super) next_id: u64,
    pub(super) jobs: VecDeque<(u64, Job)>,
}
