//! The outcome of replaying an [`EventRecording`](super::EventRecording) (spec §7.13).

/// Whether a replay ran to completion or diverged from an expected trace.
///
/// Basic replay always yields [`ReplayStatus::Completed`] — it walks the fixed,
/// recorded step list and dispatches each step, which cannot diverge. Divergence
/// is only reported by the optional checked replay
/// ([`ReplaySession::replay_checked`](super::ReplaySession::replay_checked)),
/// which flags the first step whose observed effect differs from an expected
/// recording.
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

/// A tally of what happened while replaying an
/// [`EventRecording`](super::EventRecording).
///
/// Counts are cumulative over the whole replay: how many steps were walked, how
/// many redaction markers were skipped (their payloads were dropped at record
/// time and can never be replayed, spec §7.13), how many inputs were applied,
/// how far the deterministic clock was advanced, and how many async completions
/// were dispatched — plus the final [`ReplayStatus`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplayResult {
    steps_replayed: usize,
    redactions_skipped: usize,
    inputs_applied: usize,
    clock_advanced_nanos: u64,
    completions: usize,
    status: ReplayStatus,
}

impl ReplayResult {
    /// Creates a result from its component tallies and final `status`.
    #[must_use]
    pub const fn new(
        steps_replayed: usize,
        redactions_skipped: usize,
        inputs_applied: usize,
        clock_advanced_nanos: u64,
        completions: usize,
        status: ReplayStatus,
    ) -> Self {
        Self {
            steps_replayed,
            redactions_skipped,
            inputs_applied,
            clock_advanced_nanos,
            completions,
            status,
        }
    }

    /// The number of recorded steps walked during replay (including skipped
    /// redaction markers).
    #[must_use]
    pub const fn steps_replayed(&self) -> usize {
        self.steps_replayed
    }

    /// The number of [`Redacted`](super::RecordedEvent::Redacted) markers
    /// skipped; their payloads were never recorded, so nothing was dispatched.
    #[must_use]
    pub const fn redactions_skipped(&self) -> usize {
        self.redactions_skipped
    }

    /// The number of normalized input events applied to the target.
    #[must_use]
    pub const fn inputs_applied(&self) -> usize {
        self.inputs_applied
    }

    /// The total nanoseconds the deterministic clock was advanced.
    #[must_use]
    pub const fn clock_advanced_nanos(&self) -> u64 {
        self.clock_advanced_nanos
    }

    /// The number of async completions dispatched to the target.
    #[must_use]
    pub const fn completions(&self) -> usize {
        self.completions
    }

    /// The final replay status.
    #[must_use]
    pub const fn status(&self) -> ReplayStatus {
        self.status
    }

    /// Returns `true` when replay completed without divergence.
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        matches!(self.status, ReplayStatus::Completed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors_expose_fields() {
        let result = ReplayResult::new(5, 1, 3, 42, 2, ReplayStatus::Completed);
        assert_eq!(result.steps_replayed(), 5);
        assert_eq!(result.redactions_skipped(), 1);
        assert_eq!(result.inputs_applied(), 3);
        assert_eq!(result.clock_advanced_nanos(), 42);
        assert_eq!(result.completions(), 2);
        assert_eq!(result.status(), ReplayStatus::Completed);
        assert!(result.is_complete());
    }

    #[test]
    fn diverged_is_not_complete() {
        let result = ReplayResult::new(2, 0, 1, 0, 0, ReplayStatus::Diverged { at_step: 1 });
        assert!(!result.is_complete());
        assert_eq!(result.status(), ReplayStatus::Diverged { at_step: 1 });
    }
}
