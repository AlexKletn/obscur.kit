#![doc = include_str!("replay_result.doc/_module.md")]

#[doc = include_str!("replay_result.doc/ReplayStatus.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReplayStatus {
    #[doc = include_str!("replay_result.doc/Completed.md")]
    Completed,
    #[doc = include_str!("replay_result.doc/Diverged.md")]
    Diverged {
        #[doc = include_str!("replay_result.doc/at_step.md")]
        at_step: usize,
    },
}

#[doc = include_str!("replay_result.doc/ReplayResult.md")]
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
    #[doc = include_str!("replay_result.doc/new.md")]
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

    #[doc = include_str!("replay_result.doc/steps_replayed.md")]
    #[must_use]
    pub const fn steps_replayed(&self) -> usize {
        self.steps_replayed
    }

    #[doc = include_str!("replay_result.doc/redactions_skipped.md")]
    #[must_use]
    pub const fn redactions_skipped(&self) -> usize {
        self.redactions_skipped
    }

    #[doc = include_str!("replay_result.doc/inputs_applied.md")]
    #[must_use]
    pub const fn inputs_applied(&self) -> usize {
        self.inputs_applied
    }

    #[doc = include_str!("replay_result.doc/clock_advanced_nanos.md")]
    #[must_use]
    pub const fn clock_advanced_nanos(&self) -> u64 {
        self.clock_advanced_nanos
    }

    #[doc = include_str!("replay_result.doc/completions.md")]
    #[must_use]
    pub const fn completions(&self) -> usize {
        self.completions
    }

    #[doc = include_str!("replay_result.doc/status.md")]
    #[must_use]
    pub const fn status(&self) -> ReplayStatus {
        self.status
    }

    #[doc = include_str!("replay_result.doc/is_complete.md")]
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
