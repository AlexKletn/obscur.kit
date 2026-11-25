#[doc = include_str!("../../docs/api/replay/replay_result.doc/ReplayStatus.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReplayStatus {
    /// Every recorded step was replayed without divergence.
    Completed,
    /// Replay diverged from the expected trace at the given (zero-based) step.
    Diverged {
        /// The index of the first diverging step.
        at_step: usize,
    },
}
