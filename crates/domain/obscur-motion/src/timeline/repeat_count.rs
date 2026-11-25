/// How many times a [`Timeline`](super::Timeline) repeats its iteration (spec §4.17 "chaining /
/// repeat").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RepeatCount {
    /// A fixed number of iterations. `Finite(1)` plays exactly once; `Finite(0)`
    /// requests no plays at all and a started timeline is immediately
    /// [`Finished`](TimelineState::Finished).
    Finite(u32),
    /// Repeats forever: the timeline never reaches
    /// [`Finished`](TimelineState::Finished) on its own.
    Infinite,
}
