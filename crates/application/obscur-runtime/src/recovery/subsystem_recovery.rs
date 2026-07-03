//! `SubsystemRecovery`: budgeted retry-or-degrade for non-component subsystems.

use obscur_diagnostics::DiagnosticSubsystem;

use crate::recovery::RecoveryResult;

/// Recovery for non-component subsystem failures — layout, renderer, resources
/// — which use subsystem-specific recovery boundaries (spec §1.7).
///
/// These subsystems have no component [`ErrorBoundary`](obscur_view::boundary::ErrorBoundary),
/// so recovery is coarser than component recovery: there is no fallback *view*,
/// only *retry-or-degrade*. A subsystem failure is either retried while budget
/// remains or the subsystem degrades — keeping its last-good output where it can
/// — and it never [`Propagated`](RecoveryResult::Propagated)s to a component
/// boundary.
///
/// The policy is a single retry budget shared across subsystems. Phase 1 treats
/// all subsystems uniformly; the [`DiagnosticSubsystem`] is accepted for future
/// per-subsystem policy and diagnostics attribution.
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

    /// Decides recovery for a failure in `subsystem` on retry `attempt`.
    ///
    /// Returns [`RecoveryResult::Retried`] while `attempt < max_retries` — the
    /// subsystem's work is retried — and otherwise [`RecoveryResult::FellBack`],
    /// meaning the subsystem degrades (uses its fallback / last-good output). A
    /// subsystem failure is never [`RecoveryResult::Propagated`]: there is no
    /// component boundary to escape to.
    ///
    /// `subsystem` is accepted for future per-subsystem policy and diagnostics
    /// attribution; Phase 1 treats every subsystem uniformly.
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
        assert_eq!(recovery.max_retries(), SubsystemRecovery::DEFAULT_MAX_RETRIES);
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
