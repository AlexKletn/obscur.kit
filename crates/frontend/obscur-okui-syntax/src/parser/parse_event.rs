#![doc = include_str!("parse_event.doc/_module.md")]

use crate::parser::ParseError;
use crate::syntax::{SyntaxKind, SyntaxToken};

#[doc = include_str!("parse_event.doc/ParseEvent.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum ParseEvent {
    #[doc = include_str!("parse_event.doc/StartNode.md")]
    StartNode(SyntaxKind),
    #[doc = include_str!("parse_event.doc/Token.md")]
    Token(SyntaxToken),
    #[doc = include_str!("parse_event.doc/FinishNode.md")]
    FinishNode,
    #[doc = include_str!("parse_event.doc/Error.md")]
    Error(ParseError),
}

impl ParseEvent {
    #[doc = include_str!("parse_event.doc/is_start.md")]
    #[must_use]
    pub const fn is_start(&self) -> bool {
        matches!(self, Self::StartNode(_))
    }

    #[doc = include_str!("parse_event.doc/is_finish.md")]
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
