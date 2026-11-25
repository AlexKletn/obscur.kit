/// Why the scheduler reached a recorded decision (a small typed marker).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SchedulerReason {
    /// A frame was scheduled.
    Frame,
    /// A timer fired.
    Timer,
    /// Idle work was scheduled.
    Idle,
    /// Work was coalesced with pending work of the same kind.
    Coalesced,
}
