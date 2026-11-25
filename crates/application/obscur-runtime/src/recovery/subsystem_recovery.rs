//! `SubsystemRecovery`: budgeted retry-or-degrade for non-component subsystems.

use obscur_diagnostics::DiagnosticSubsystem;

use crate::recovery::RecoveryResult;

#[doc = include_str!("../../docs/api/recovery/subsystem_recovery.doc/SubsystemRecovery.md")]
#[derive(Debug, Clone, Copy)]
pub struct SubsystemRecovery {
    /// How many times a subsystem's work may be retried before it degrades.
    max_retries: u32,
}

impl SubsystemRecovery {
    /// The default subsystem retry budget: a single retry before degrading.
    pub const DEFAULT_MAX_RETRIES: u32 = 1;

    /// Creates a subsystem-recovery policy with a retry budget of `max_retries`.
    #[must_use]
    pub const fn new(max_retries: u32) -> Self {
        Self { max_retries }
    }

    /// The configured retry budget.
    #[must_use]
    pub const fn max_retries(self) -> u32 {
        self.max_retries
    }

    #[doc = include_str!("../../docs/api/recovery/subsystem_recovery.doc/recover.md")]
    #[must_use]
    pub const fn recover(self, subsystem: DiagnosticSubsystem, attempt: u32) -> RecoveryResult {
        let _ = subsystem;
        if attempt < self.max_retries {
            RecoveryResult::Retried
        } else {
            RecoveryResult::FellBack
        }
    }
}

impl Default for SubsystemRecovery {
    /// A policy with a small default budget ([`DEFAULT_MAX_RETRIES`](Self::DEFAULT_MAX_RETRIES)).
    fn default() -> Self {
        Self::new(Self::DEFAULT_MAX_RETRIES)
    }
}

#[cfg(test)]
mod tests {
    use super::SubsystemRecovery;
    use crate::recovery::RecoveryResult;
    use obscur_diagnostics::DiagnosticSubsystem;

    #[test]
    fn retries_while_under_budget() {
        let recovery = SubsystemRecovery::new(2);
        assert_eq!(
            recovery.recover(DiagnosticSubsystem::Layout, 0),
            RecoveryResult::Retried
        );
        assert_eq!(
            recovery.recover(DiagnosticSubsystem::Render, 1),
            RecoveryResult::Retried
        );
    }

    #[test]
    fn degrades_once_budget_reached() {
        let recovery = SubsystemRecovery::new(2);
        assert_eq!(
            recovery.recover(DiagnosticSubsystem::Resources, 2),
            RecoveryResult::FellBack
        );
        assert_eq!(
            recovery.recover(DiagnosticSubsystem::Resources, 3),
            RecoveryResult::FellBack
        );
    }

    #[test]
    fn default_has_small_budget() {
        let recovery = SubsystemRecovery::default();
        assert_eq!(
            recovery.max_retries(),
            SubsystemRecovery::DEFAULT_MAX_RETRIES
        );
        // With a budget of 1, attempt 0 retries and attempt 1 degrades.
        assert_eq!(
            recovery.recover(DiagnosticSubsystem::Layout, 0),
            RecoveryResult::Retried
        );
        assert_eq!(
            recovery.recover(DiagnosticSubsystem::Layout, 1),
            RecoveryResult::FellBack
        );
    }

    #[test]
    fn subsystem_does_not_affect_decision() {
        let recovery = SubsystemRecovery::new(1);
        // Phase 1 treats all subsystems uniformly.
        assert_eq!(
            recovery.recover(DiagnosticSubsystem::Layout, 0),
            recovery.recover(DiagnosticSubsystem::Render, 0)
        );
    }
}
