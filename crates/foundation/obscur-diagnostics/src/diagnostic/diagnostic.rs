#![doc = include_str!("diagnostic.doc/_module.md")]

use super::{
    DiagnosticCode, DiagnosticContext, DiagnosticSeverity, DiagnosticSubsystem, RecoveryAction,
};

#[doc = include_str!("diagnostic.doc/Diagnostic.md")]
#[derive(Debug, Clone)]
pub struct Diagnostic {
    #[doc = include_str!("diagnostic.doc/code.md")]
    pub code: DiagnosticCode,
    #[doc = include_str!("diagnostic.doc/severity.md")]
    pub severity: DiagnosticSeverity,
    #[doc = include_str!("diagnostic.doc/subsystem.md")]
    pub subsystem: DiagnosticSubsystem,
    #[doc = include_str!("diagnostic.doc/message.md")]
    pub message: String,
    #[doc = include_str!("diagnostic.doc/context.md")]
    pub context: DiagnosticContext,
    #[doc = include_str!("diagnostic.doc/recovery.md")]
    pub recovery: RecoveryAction,
}

impl Diagnostic {
    #[doc = include_str!("diagnostic.doc/new.md")]
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

    #[doc = include_str!("diagnostic.doc/with_recovery.md")]
    #[must_use]
    pub const fn with_recovery(mut self, recovery: RecoveryAction) -> Self {
        self.recovery = recovery;
        self
    }
}
