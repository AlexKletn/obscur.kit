#![doc = include_str!("blocking_executor.doc/_module.md")]

#[doc = include_str!("blocking_executor.doc/BlockingExecutor.md")]
pub trait BlockingExecutor {
    #[doc = include_str!("blocking_executor.doc/run_blocking.md")]
    fn run_blocking(&self, job: Box<dyn FnOnce() + Send + 'static>);
}
