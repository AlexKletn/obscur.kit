//! Spawner port for thread-local (`!Send`) futures.

use std::future::Future;
use std::pin::Pin;

/// A boxed, non-`Send` future pinned to the UI thread.
pub type LocalBoxFuture = Pin<Box<dyn Future<Output = ()> + 'static>>;

/// Schedules non-`Send` work on the owning UI thread.
pub trait LocalSpawner {
    /// Spawns a thread-local future for execution.
    fn spawn_local(&self, future: LocalBoxFuture);
}
