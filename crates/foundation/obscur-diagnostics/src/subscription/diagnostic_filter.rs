//! Filter applied to a diagnostic subscription.

use crate::diagnostic::{Diagnostic, DiagnosticSeverity};

/// A minimum-severity filter for a diagnostic subscription (spec §7.8).
#[derive(Debug, Clone, Copy)]
pub struct DiagnosticFilter {
    /// Lowest severity that passes the filter.
    pub min_severity: DiagnosticSeverity,
}

impl DiagnosticFilter {
    /// Creates a filter admitting diagnostics at or above `min_severity`.
    #[must_use]
    pub const fn min(min_severity: DiagnosticSeverity) -> Self {
        Self { min_severity }
    }

    /// Returns whether `diagnostic` passes this filter.
    #[must_use]
    pub fn admits(&self, diagnostic: &Diagnostic) -> bool {
        diagnostic.severity >= self.min_severity
    }
}
