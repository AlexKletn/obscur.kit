//! `ErrorBoundary`: the mechanism the runtime consults to contain failures.

use crate::boundary::{ComponentFailure, ErrorBoundaryPolicy};
use crate::view::View;

/// A component's error boundary (spec §1.7).
///
/// This is the mechanism the runtime consults when it detects a contained
/// failure. It pairs a [`ErrorBoundaryPolicy`] with the current
/// [`ComponentFailure`] (if any) and an optional fallback [`View`] to show in
/// place of the failed subtree.
///
/// The core rule is *contain vs. propagate*: when the policy is enabled a
/// recorded failure is contained here — the subtree is replaced by the fallback
/// UI. When the policy is disabled the failure is still recorded (so it is
/// reported to diagnostics), but [`record`](Self::record) signals that this
/// boundary does not contain it, and the runtime must propagate it to the
/// nearest enclosing enabled boundary.
///
/// There is no reactive engine and no panic-catching here: the runtime feeds
/// failures in and reads the containment decision and fallback back out.
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

    /// Records `failure` as the boundary's current failure and returns whether
    /// this boundary *contains* it.
    ///
    /// Returns `true` when the policy is enabled — the failure is contained and
    /// the subtree is now to be replaced by the [`fallback_view`](Self::fallback_view).
    /// Returns `false` when the policy is disabled — the failure is stored (so
    /// it is still reported) but must be propagated to an outer boundary.
    pub fn record(&mut self, failure: ComponentFailure) -> bool {
        self.failure = Some(failure);
        self.policy.is_enabled()
    }

    /// Returns whether a contained failure may still be retried.
    ///
    /// A retry is available only while the boundary is enabled and the current
    /// failure's attempt count is below the policy's budget. With no recorded
    /// failure there is nothing to retry.
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

    /// Clears the current failure, returning the boundary to its healthy state.
    ///
    /// The fallback view and policy are left untouched.
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
