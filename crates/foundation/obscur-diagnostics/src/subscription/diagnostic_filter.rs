#![doc = include_str!("diagnostic_filter.doc/_module.md")]

use crate::diagnostic::{Diagnostic, DiagnosticSeverity};

#[doc = include_str!("diagnostic_filter.doc/DiagnosticFilter.md")]
#[derive(Debug, Clone, Copy)]
pub struct DiagnosticFilter {
    #[doc = include_str!("diagnostic_filter.doc/min_severity.md")]
    pub min_severity: DiagnosticSeverity,
}

impl DiagnosticFilter {
    #[doc = include_str!("diagnostic_filter.doc/min.md")]
    #[must_use]
    pub const fn min(min_severity: DiagnosticSeverity) -> Self {
        Self { min_severity }
    }

    #[doc = include_str!("diagnostic_filter.doc/admits.md")]
    #[must_use]
    pub fn admits(&self, diagnostic: &Diagnostic) -> bool {
        diagnostic.severity >= self.min_severity
    }
}
