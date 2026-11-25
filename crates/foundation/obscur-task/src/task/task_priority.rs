//! Scheduling priority of a task.

/// Coarse scheduling priority (spec §3.6). Lower-priority work must not starve,
/// but yields to input and commit work.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    /// Background maintenance (caches, telemetry).
    Background,
    /// Normal application work.
    Normal,
    /// Latency-sensitive work adjacent to input.
    Interactive,
}
