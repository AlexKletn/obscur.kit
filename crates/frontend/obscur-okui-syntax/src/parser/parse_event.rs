//! `ParseEvent`: the flat, rowan-style event stream the parser emits before
//! materializing it into a [`SyntaxTree`](crate::syntax::SyntaxTree).

use crate::parser::ParseError;
use crate::syntax::{SyntaxKind, SyntaxToken};

#[doc = include_str!("../../docs/api/parser/parse_event.doc/ParseEvent.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum ParseEvent {
    /// Open a new interior node of the given (node-valued) kind. Subsequent
    /// tokens and child nodes attach to it until the matching
    /// [`FinishNode`](Self::FinishNode).
    StartNode(SyntaxKind),
    /// Attach a leaf token to the currently open node.
    Token(SyntaxToken),
    /// Close the most recently opened, still-open node.
    FinishNode,
    /// Record a diagnostic. Does not open or close a node.
    Error(ParseError),
}

impl ParseEvent {
    /// Returns `true` if this event opens a node.
    #[must_use]
    pub const fn is_start(&self) -> bool {
        matches!(self, Self::StartNode(_))
    }

    /// Returns `true` if this event closes a node.
    #[must_use]
    pub const fn is_finish(&self) -> bool {
        matches!(self, Self::FinishNode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ParseErrorKind;

    #[test]
    fn is_start_only_for_start_node() {
        let ev = ParseEvent::StartNode(SyntaxKind::Document);
        assert!(ev.is_start());
        assert!(!ev.is_finish());
    }

    #[test]
    fn is_finish_only_for_finish_node() {
        assert!(ParseEvent::FinishNode.is_finish());
        assert!(!ParseEvent::FinishNode.is_start());
    }

    #[test]
    fn token_and_error_are_neither_start_nor_finish() {
        let tok = ParseEvent::Token(SyntaxToken::new(SyntaxKind::Ident, "x", 0));
        assert!(!tok.is_start());
        assert!(!tok.is_finish());

        let err = ParseEvent::Error(ParseError::new(ParseErrorKind::UnexpectedToken, "m", 0));
        assert!(!err.is_start());
        assert!(!err.is_finish());
    }
}
