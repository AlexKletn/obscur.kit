use crate::source::SourceSpan;

use super::css_token_kind::CssTokenKind;

/// A single CSS token: a [`CssTokenKind`] together with the [`SourceSpan`] it
/// occupies in the originating source.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CssToken {
    kind: CssTokenKind,
    span: SourceSpan,
}

impl CssToken {
    /// Creates a token of `kind` covering `span`.
    pub(crate) fn new(kind: CssTokenKind, span: SourceSpan) -> Self {
        Self { kind, span }
    }

    /// Returns the token's lexical kind.
    pub(crate) fn kind(&self) -> &CssTokenKind {
        &self.kind
    }

    /// Returns the span the token occupies.
    pub(crate) fn span(&self) -> SourceSpan {
        self.span
    }

    /// Returns whether this token is whitespace.
    pub(crate) fn is_whitespace(&self) -> bool {
        matches!(self.kind, CssTokenKind::Whitespace)
    }
}
