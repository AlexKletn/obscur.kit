/// What servicing one unit left behind, and therefore what the scheduler must be
/// told next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BackgroundServiced {
    /// Units remain queued: later frames still count as preemptions of them.
    More,
    /// That was the last unit. The scheduler must stop treating later frames as
    /// preemptions, or it goes on reporting starvation for work that no longer
    /// exists.
    Drained,
}
