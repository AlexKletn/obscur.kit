//! A structured diagnostic record.

use super::{
    DiagnosticCode, DiagnosticContext, DiagnosticSeverity, DiagnosticSubsystem, RecoveryAction,
};

/// A structured, typed diagnostic — not a plain log line (spec §1.6, §7.8).
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Stable diagnostic code.
    pub code: DiagnosticCode,
    /// Severity.
    pub severity: DiagnosticSeverity,
    /// Originating subsystem.
    pub subsystem: DiagnosticSubsystem,
    /// Human-readable message.
    pub message: String,
    /// Identity context.
    pub context: DiagnosticContext,
    /// Recovery action taken or recommended.
    pub recovery: RecoveryAction,
}

impl Diagnostic {
    /// Creates a diagnostic with empty context and no recovery action.
    #[must_use]
    pub fn new(
        code: DiagnosticCode,
        severity: DiagnosticSeverity,
        subsystem: DiagnosticSubsystem,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            severity,
            subsystem,
            message: message.into(),
            context: DiagnosticContext::default(),
            recovery: RecoveryAction::None,
        }
    }
}
