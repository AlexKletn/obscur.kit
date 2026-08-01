#![doc = include_str!("severity.doc/_module.md")]

#[doc = include_str!("severity.doc/LspSeverity.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LspSeverity {
    #[doc = include_str!("severity.doc/Error.md")]
    Error,
    #[doc = include_str!("severity.doc/Warning.md")]
    Warning,
    #[doc = include_str!("severity.doc/Information.md")]
    Information,
    #[doc = include_str!("severity.doc/Hint.md")]
    Hint,
}

impl LspSeverity {
    #[doc = include_str!("severity.doc/code.md")]
    #[must_use]
    pub const fn code(self) -> u8 {
        match self {
            Self::Error => 1,
            Self::Warning => 2,
            Self::Information => 3,
            Self::Hint => 4,
        }
    }

    #[doc = include_str!("severity.doc/label.md")]
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
