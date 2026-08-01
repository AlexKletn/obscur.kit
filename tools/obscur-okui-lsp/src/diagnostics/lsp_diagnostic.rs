#![doc = include_str!("lsp_diagnostic.doc/_module.md")]

use crate::diagnostics::{LspSeverity, Range};

#[doc = include_str!("lsp_diagnostic.doc/LspDiagnostic.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspDiagnostic {
    #[doc = include_str!("lsp_diagnostic.doc/range.md")]
    pub range: Range,
    #[doc = include_str!("lsp_diagnostic.doc/severity.md")]
    pub severity: LspSeverity,
    #[doc = include_str!("lsp_diagnostic.doc/message.md")]
    pub message: Box<str>,
    #[doc = include_str!("lsp_diagnostic.doc/source.md")]
    pub source: &'static str,
}

impl LspDiagnostic {
    #[doc = include_str!("lsp_diagnostic.doc/new.md")]
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
