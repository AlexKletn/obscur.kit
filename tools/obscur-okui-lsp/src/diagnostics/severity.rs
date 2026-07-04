//! `LspSeverity`: the severity of an [`LspDiagnostic`](crate::LspDiagnostic),
//! mirroring the LSP `DiagnosticSeverity` enumeration.

/// The severity of a diagnostic, matching the LSP `DiagnosticSeverity` values
/// (`Error = 1`, `Warning = 2`, `Information = 3`, `Hint = 4`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LspSeverity {
    /// A hard error: the source is invalid.
    Error,
    /// A warning: suspicious but not fatal.
    Warning,
    /// Informational feedback.
    Information,
    /// A gentle hint.
    Hint,
}

impl LspSeverity {
    /// Returns the LSP wire code for this severity (`Error = 1` … `Hint = 4`).
    #[must_use]
    pub const fn code(self) -> u8 {
        match self {
            Self::Error => 1,
            Self::Warning => 2,
            Self::Information => 3,
            Self::Hint => 4,
        }
    }

    /// Returns a short lowercase label for CLI output.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Information => "info",
            Self::Hint => "hint",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_match_lsp_wire_values() {
        assert_eq!(LspSeverity::Error.code(), 1);
        assert_eq!(LspSeverity::Warning.code(), 2);
        assert_eq!(LspSeverity::Information.code(), 3);
        assert_eq!(LspSeverity::Hint.code(), 4);
    }

    #[test]
    fn labels_are_stable() {
        assert_eq!(LspSeverity::Error.label(), "error");
        assert_eq!(LspSeverity::Warning.label(), "warning");
    }
}
