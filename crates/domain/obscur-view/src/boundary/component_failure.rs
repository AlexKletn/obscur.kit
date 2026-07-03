//! `ComponentFailure`: the structured record of a contained component failure.

use obscur_diagnostics::{Diagnostic, DiagnosticCode, DiagnosticSeverity, DiagnosticSubsystem};

use crate::lifecycle::LifecycleState;

/// A structured record of a failure the runtime detected inside a component
/// (spec §1.7).
///
/// This is *not* panic-catching: native Rust is trusted (spec §1.8), so there
/// is no `catch_unwind` here. It is the honest, structured value the runtime
/// produces when it detects a contained failure in view generation,
/// reconciliation, or a local lifecycle hook — carrying the
/// [`LifecycleState`] the failure happened in, the retry attempt that produced
/// it, and a [`Diagnostic`] (subsystem [`DiagnosticSubsystem::View`]) describing
/// it for diagnostics and `DevTools`.
#[derive(Debug, Clone)]
pub struct ComponentFailure {
    /// The lifecycle phase the failure occurred in.
    phase: LifecycleState,
    /// Which retry attempt produced this failure (0 for the first occurrence).
    attempt: u32,
    /// The structured diagnostic describing the failure.
    diagnostic: Diagnostic,
}

impl ComponentFailure {
    /// Records a failure in `phase` on retry `attempt`, building an `Error`
    /// [`Diagnostic`] in the [`View`](DiagnosticSubsystem::View) subsystem from
    /// `code` and `message`.
    #[must_use]
    pub fn new(
        phase: LifecycleState,
        attempt: u32,
        code: DiagnosticCode,
        message: impl Into<String>,
    ) -> Self {
        let diagnostic = Diagnostic::new(
            code,
            DiagnosticSeverity::Error,
            DiagnosticSubsystem::View,
            message,
        );
        Self {
            phase,
            attempt,
            diagnostic,
        }
    }

    /// Records a failure in `phase` on retry `attempt` from an already-built
    /// [`Diagnostic`].
    #[must_use]
    pub fn from_diagnostic(phase: LifecycleState, attempt: u32, diagnostic: Diagnostic) -> Self {
        Self {
            phase,
            attempt,
            diagnostic,
        }
    }

    /// The lifecycle phase the failure occurred in.
    #[must_use]
    pub const fn phase(&self) -> LifecycleState {
        self.phase
    }

    /// The retry attempt that produced this failure.
    #[must_use]
    pub const fn attempt(&self) -> u32 {
        self.attempt
    }

    /// The structured diagnostic describing the failure.
    #[must_use]
    pub const fn diagnostic(&self) -> &Diagnostic {
        &self.diagnostic
    }
}

#[cfg(test)]
mod tests {
    use super::ComponentFailure;
    use crate::lifecycle::LifecycleState;
    use obscur_diagnostics::{DiagnosticCode, DiagnosticSeverity, DiagnosticSubsystem};

    #[test]
    fn new_builds_view_error_diagnostic() {
        let failure = ComponentFailure::new(
            LifecycleState::Updating,
            2,
            DiagnosticCode::new("view.component.failed"),
            "render produced an invalid tree",
        );
        assert_eq!(failure.phase(), LifecycleState::Updating);
        assert_eq!(failure.attempt(), 2);
        assert_eq!(failure.diagnostic().severity, DiagnosticSeverity::Error);
        assert_eq!(failure.diagnostic().subsystem, DiagnosticSubsystem::View);
        assert_eq!(failure.diagnostic().code.as_str(), "view.component.failed");
    }
}
