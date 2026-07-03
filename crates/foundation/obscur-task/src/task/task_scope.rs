//! Ownership scope of a task.

/// The scope a task is bound to; when the scope ends the task is cancelled
/// automatically (spec §3.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskScope {
    /// Bound to a component instance.
    Component,
    /// Bound to a window.
    Window,
    /// Bound to the runtime.
    Runtime,
}
