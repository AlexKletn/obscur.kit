//! `ErrorBoundary`: the mechanism the runtime consults to contain failures.

use crate::boundary::{ComponentFailure, ErrorBoundaryPolicy};
use crate::view::View;

#[doc = include_str!("../../docs/api/boundary/error_boundary.doc/ErrorBoundary.md")]
#[derive(Debug, Default)]
pub struct ErrorBoundary {
    policy: ErrorBoundaryPolicy,
    failure: Option<ComponentFailure>,
    fallback: Option<View>,
}

impl ErrorBoundary {
    /// Creates a boundary governed by `policy`, with no active failure and no
    /// fallback view yet.
    #[must_use]
    pub fn new(policy: ErrorBoundaryPolicy) -> Self {
        Self {
            policy,
            failure: None,
            fallback: None,
        }
    }

    #[doc = include_str!("../../docs/api/boundary/error_boundary.doc/record.md")]
    pub fn record(&mut self, failure: ComponentFailure) -> bool {
        self.failure = Some(failure);
        self.policy.is_enabled()
    }

    #[doc = include_str!("../../docs/api/boundary/error_boundary.doc/should_retry.md")]
    #[must_use]
    pub fn should_retry(&self) -> bool {
        if !self.policy.is_enabled() {
            return false;
        }
        match &self.failure {
            Some(failure) => failure.attempt() < self.policy.max_retries,
            None => false,
        }
    }

    #[doc = include_str!("../../docs/api/boundary/error_boundary.doc/reset.md")]
    pub fn reset(&mut self) {
        self.failure = None;
    }

    /// The current failure, if the boundary is holding one.
    #[must_use]
    pub fn failure(&self) -> Option<&ComponentFailure> {
        self.failure.as_ref()
    }

    /// Whether a failure is currently contained by this boundary.
    #[must_use]
    pub fn is_tripped(&self) -> bool {
        self.failure.is_some()
    }

    /// The governing policy.
    #[must_use]
    pub const fn policy(&self) -> ErrorBoundaryPolicy {
        self.policy
    }

    /// The fallback view shown in place of a failed subtree, if one is set.
    #[must_use]
    pub fn fallback_view(&self) -> Option<&View> {
        self.fallback.as_ref()
    }

    /// Sets the fallback view shown when a failure is contained.
    pub fn set_fallback(&mut self, fallback: View) {
        self.fallback = Some(fallback);
    }
}

#[cfg(test)]
mod tests {
    use super::ErrorBoundary;
    use crate::boundary::{ComponentFailure, ErrorBoundaryPolicy};
    use crate::lifecycle::LifecycleState;
    use crate::view::View;
    use obscur_diagnostics::DiagnosticCode;

    fn failure(attempt: u32) -> ComponentFailure {
        ComponentFailure::new(
            LifecycleState::Mounted,
            attempt,
            DiagnosticCode::new("view.test.failed"),
            "boom",
        )
    }

    #[test]
    fn enabled_boundary_contains_failure() {
        let mut boundary = ErrorBoundary::new(ErrorBoundaryPolicy::default());
        assert!(boundary.record(failure(0)));
        assert!(boundary.is_tripped());
    }

    #[test]
    fn disabled_boundary_stores_but_propagates() {
        let mut boundary = ErrorBoundary::new(ErrorBoundaryPolicy::disabled());
        assert!(!boundary.record(failure(0)));
        // Still stored so it can be reported to diagnostics.
        assert!(boundary.is_tripped());
        assert!(!boundary.should_retry());
    }

    #[test]
    fn should_retry_respects_budget() {
        let policy = ErrorBoundaryPolicy::default().with_max_retries(2);
        let mut boundary = ErrorBoundary::new(policy);
        boundary.record(failure(0));
        assert!(boundary.should_retry());
        boundary.record(failure(2));
        assert!(!boundary.should_retry());
    }

    #[test]
    fn reset_clears_failure() {
        let mut boundary = ErrorBoundary::new(ErrorBoundaryPolicy::default());
        boundary.record(failure(0));
        boundary.reset();
        assert!(!boundary.is_tripped());
        assert!(boundary.failure().is_none());
    }

    #[test]
    fn fallback_round_trips() {
        let mut boundary = ErrorBoundary::default();
        assert!(boundary.fallback_view().is_none());
        boundary.set_fallback(View::empty());
        assert!(boundary.fallback_view().is_some());
    }
}
