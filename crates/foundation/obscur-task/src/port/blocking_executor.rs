//! Blocking-work port.

/// Runs an explicitly blocking job off the UI thread. Blocking work is opt-in
/// and capability-dependent (spec §3.3).
pub trait BlockingExecutor {
    /// Submits a blocking job for execution.
    fn run_blocking(&self, job: Box<dyn FnOnce() + Send + 'static>);
}
