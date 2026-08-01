#![doc = include_str!("component_recovery.doc/_module.md")]

use obscur_view::boundary::{ComponentFailure, ErrorBoundary};

use crate::recovery::RecoveryResult;

#[doc = include_str!("component_recovery.doc/ComponentRecovery.md")]
#[derive(Debug, Clone, Copy, Default)]
pub struct ComponentRecovery;

impl ComponentRecovery {
    #[doc = include_str!("component_recovery.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[doc = include_str!("component_recovery.doc/recover.md")]
    #[must_use]
    pub fn recover(
        &self,
        boundary: &mut ErrorBoundary,
        failure: ComponentFailure,
    ) -> RecoveryResult {
        let contained = boundary.record(failure);
        if !contained {
            RecoveryResult::Propagated
        } else if boundary.should_retry() {
            RecoveryResult::Retried
        } else {
            RecoveryResult::FellBack
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ComponentRecovery;
    use crate::recovery::RecoveryResult;
    use obscur_diagnostics::DiagnosticCode;
    use obscur_view::boundary::{ComponentFailure, ErrorBoundary, ErrorBoundaryPolicy};
    use obscur_view::lifecycle::LifecycleState;

    fn failure(attempt: u32) -> ComponentFailure {
        ComponentFailure::new(
            LifecycleState::Mounted,
            attempt,
            DiagnosticCode::new("view.component.failed"),
            "boom",
        )
    }

    #[test]
    fn enabled_boundary_retries_while_budget_remains() {
        let recovery = ComponentRecovery::new();
        let mut boundary = ErrorBoundary::new(ErrorBoundaryPolicy::default());
        // Default budget is 1, so the first failure (attempt 0) can still retry.
        assert_eq!(
            recovery.recover(&mut boundary, failure(0)),
            RecoveryResult::Retried
        );
        assert!(boundary.is_tripped());
    }

    #[test]
    fn enabled_boundary_falls_back_once_budget_exhausted() {
        let recovery = ComponentRecovery::new();
        let policy = ErrorBoundaryPolicy::default().with_max_retries(2);
        let mut boundary = ErrorBoundary::new(policy);
        // attempt < max_retries → retry.
        assert_eq!(
            recovery.recover(&mut boundary, failure(0)),
            RecoveryResult::Retried
        );
        assert_eq!(
            recovery.recover(&mut boundary, failure(1)),
            RecoveryResult::Retried
        );
        // attempt == max_retries → out of budget → fallback kept.
        assert_eq!(
            recovery.recover(&mut boundary, failure(2)),
            RecoveryResult::FellBack
        );
    }

    #[test]
    fn disabled_boundary_propagates_but_still_records() {
        let recovery = ComponentRecovery;
        let mut boundary = ErrorBoundary::new(ErrorBoundaryPolicy::disabled());
        assert_eq!(
            recovery.recover(&mut boundary, failure(0)),
            RecoveryResult::Propagated
        );
        // Recorded so diagnostics still sees it, even though it propagates.
        assert!(boundary.is_tripped());
    }
}
