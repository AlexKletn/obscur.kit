//! `ComponentRecovery`: applies the §1.7 component error-boundary policy.

use obscur_view::boundary::{ComponentFailure, ErrorBoundary};

use crate::recovery::RecoveryResult;

/// Applies the component error-boundary policy (spec §1.7).
///
/// Every component is an error boundary by default: a failure in view
/// generation, reconciliation, or a local lifecycle hook is *contained* within
/// the component subtree, may be *retried* according to policy, and otherwise
/// the subtree is replaced with fallback UI — unless the boundary is disabled,
/// in which case the failure *propagates* to the nearest enclosing boundary.
///
/// This coordinator is stateless: it sequences the §1.7
/// *contain → (retry | fallback) | propagate* decision, delegating the
/// contain/retry policy itself to `obscur-view`'s [`ErrorBoundary`]. The runtime
/// only orders the decision and (later) reports it to diagnostics and reruns or
/// replaces the subtree; it does not reimplement the boundary here.
#[derive(Debug, Clone, Copy, Default)]
pub struct ComponentRecovery;

impl ComponentRecovery {
    /// Creates a component-recovery coordinator.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Applies the §1.7 error-boundary decision for `failure` against
    /// `boundary`, returning the [`RecoveryResult`].
    ///
    /// The sequence is:
    /// 1. Record the failure in the boundary. It is *always* recorded — even a
    ///    disabled boundary stores it — so it can be reported to diagnostics.
    /// 2. If the boundary did not contain it (disabled), the failure escapes:
    ///    [`RecoveryResult::Propagated`].
    /// 3. Otherwise, if the boundary still has retry budget, retry:
    ///    [`RecoveryResult::Retried`].
    /// 4. Otherwise the fallback UI is kept in place of the failed subtree:
    ///    [`RecoveryResult::FellBack`].
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
    use obscur_view::boundary::{ComponentFailure, ErrorBoundary, ErrorBoundaryPolicy};
    use obscur_view::lifecycle::LifecycleState;
    use obscur_diagnostics::DiagnosticCode;

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
        assert_eq!(recovery.recover(&mut boundary, failure(0)), RecoveryResult::Retried);
        assert!(boundary.is_tripped());
    }

    #[test]
    fn enabled_boundary_falls_back_once_budget_exhausted() {
        let recovery = ComponentRecovery::new();
        let policy = ErrorBoundaryPolicy::default().with_max_retries(2);
        let mut boundary = ErrorBoundary::new(policy);
        // attempt < max_retries → retry.
        assert_eq!(recovery.recover(&mut boundary, failure(0)), RecoveryResult::Retried);
        assert_eq!(recovery.recover(&mut boundary, failure(1)), RecoveryResult::Retried);
        // attempt == max_retries → out of budget → fallback kept.
        assert_eq!(recovery.recover(&mut boundary, failure(2)), RecoveryResult::FellBack);
    }

    #[test]
    fn disabled_boundary_propagates_but_still_records() {
        let recovery = ComponentRecovery::default();
        let mut boundary = ErrorBoundary::new(ErrorBoundaryPolicy::disabled());
        assert_eq!(recovery.recover(&mut boundary, failure(0)), RecoveryResult::Propagated);
        // Recorded so diagnostics still sees it, even though it propagates.
        assert!(boundary.is_tripped());
    }
}
