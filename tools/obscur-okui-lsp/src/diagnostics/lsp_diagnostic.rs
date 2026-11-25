//! `LspDiagnostic`: a transport-agnostic diagnostic ready to be published to an
//! LSP client.

use crate::diagnostics::{LspSeverity, Range};

#[doc = include_str!("../../docs/api/diagnostics/lsp_diagnostic.doc/LspDiagnostic.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspDiagnostic {
    /// The source span the diagnostic applies to.
    pub range: Range,
    /// The severity of the diagnostic.
    pub severity: LspSeverity,
    /// The human-readable message.
    pub message: Box<str>,
    /// A short label naming the pass that produced the diagnostic
    /// (e.g. `"okui-parse"`, `"okui-compile"`).
    pub source: &'static str,
}

impl LspDiagnostic {
    /// Creates a diagnostic over `range` with `severity`, `message` and
    /// `source`.
    #[must_use]
    pub fn new(
        range: Range,
        severity: LspSeverity,
        message: impl Into<Box<str>>,
        source: &'static str,
    ) -> Self {
        Self {
            range,
            severity,
            message: message.into(),
            source,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Position;

    #[test]
    fn stores_all_fields() {
        let range = Range::new(Position::new(1, 0), Position::new(1, 4));
        let diagnostic = LspDiagnostic::new(range, LspSeverity::Error, "boom", "okui-parse");
        assert_eq!(diagnostic.range, range);
        assert_eq!(diagnostic.severity, LspSeverity::Error);
        assert_eq!(&*diagnostic.message, "boom");
        assert_eq!(diagnostic.source, "okui-parse");
    }
}
