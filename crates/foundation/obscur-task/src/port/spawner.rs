//! Spawner port for `Send` futures.

use std::future::Future;
use std::pin::Pin;

/// A boxed, `Send` future scheduled onto an executor adapter.
pub type BoxFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

/// Schedules `Send` work onto the platform executor. The core never depends on
/// a concrete executor (spec §3.3, §4.8).
pub trait Spawner {
    /// Spawns a future for execution.
    fn spawn(&self, future: BoxFuture);
}
