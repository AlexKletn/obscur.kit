#![doc = include_str!("component_failure.doc/_module.md")]

use obscur_diagnostics::{Diagnostic, DiagnosticCode, DiagnosticSeverity, DiagnosticSubsystem};

use crate::lifecycle::LifecycleState;

#[doc = include_str!("component_failure.doc/ComponentFailure.md")]
#[derive(Debug, Clone)]
pub struct ComponentFailure {
    #[doc = include_str!("component_failure.doc/phase.md")]
    phase: LifecycleState,
    #[doc = include_str!("component_failure.doc/attempt.md")]
    attempt: u32,
    #[doc = include_str!("component_failure.doc/diagnostic.md")]
    diagnostic: Diagnostic,
}

impl ComponentFailure {
    #[doc = include_str!("component_failure.doc/new.md")]
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

    #[doc = include_str!("component_failure.doc/from_diagnostic.md")]
    #[must_use]
    pub fn from_diagnostic(phase: LifecycleState, attempt: u32, diagnostic: Diagnostic) -> Self {
        Self {
            phase,
            attempt,
            diagnostic,
        }
    }

    #[doc = include_str!("component_failure.doc/phase_2.md")]
    #[must_use]
    pub const fn phase(&self) -> LifecycleState {
        self.phase
    }

    #[doc = include_str!("component_failure.doc/attempt_2.md")]
    #[must_use]
    pub const fn attempt(&self) -> u32 {
        self.attempt
    }

    #[doc = include_str!("component_failure.doc/diagnostic_2.md")]
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
