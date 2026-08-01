#![doc = include_str!("token_cursor.doc/_module.md")]

use crate::source::SourceSpan;

use super::css_tokenizer::{CssToken, CssTokenKind};

#[doc = include_str!("token_cursor.doc/TokenCursor.md")]
pub(crate) struct TokenCursor<'t> {
    tokens: &'t [CssToken],
    pos: usize,
    eof: u32,
}

impl<'t> TokenCursor<'t> {
    #[doc = include_str!("token_cursor.doc/new.md")]
    pub(crate) fn new(tokens: &'t [CssToken], eof: u32) -> Self {
        Self {
            tokens,
            pos: 0,
            eof,
        }
    }

    #[doc = include_str!("token_cursor.doc/peek.md")]
    pub(crate) fn peek(&self) -> Option<&'t CssToken> {
        self.tokens.get(self.pos)
    }

    #[doc = include_str!("token_cursor.doc/peek_kind.md")]
    pub(crate) fn peek_kind(&self) -> Option<&'t CssTokenKind> {
        self.tokens.get(self.pos).map(CssToken::kind)
    }

    #[doc = include_str!("token_cursor.doc/advance.md")]
    pub(crate) fn advance(&mut self) -> Option<&'t CssToken> {
        let tok = self.tokens.get(self.pos);
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    #[doc = include_str!("token_cursor.doc/skip_ws.md")]
    pub(crate) fn skip_ws(&mut self) {
        while matches!(self.tokens.get(self.pos), Some(t) if t.is_whitespace()) {
            self.pos += 1;
        }
    }

    #[doc = include_str!("token_cursor.doc/eof.md")]
    pub(crate) fn eof(&self) -> u32 {
        self.eof
    }

    #[doc = include_str!("token_cursor.doc/here_span.md")]
    pub(crate) fn here_span(&self) -> SourceSpan {
        self.tokens
            .get(self.pos)
            .map_or_else(|| SourceSpan::empty(self.eof), CssToken::span)
    }
}

#[doc = include_str!("token_cursor.doc/slice_span.md")]
pub(crate) fn slice_span(tokens: &[CssToken], eof: u32) -> SourceSpan {
    match (tokens.first(), tokens.last()) {
        (Some(first), Some(last)) => first.span().union(last.span()),
        _ => SourceSpan::empty(eof),
    }
}

#[cfg(test)]
mod tests {
    use super::super::css_tokenizer::Tokenizer;
    use super::*;

    fn toks(text: &str) -> Vec<CssToken> {
        Tokenizer::new(text).tokenize().0
    }

    #[test]
    fn advance_walks_the_slice_and_stops_at_the_end() {
        let tokens = toks("a b");
        let mut cursor = TokenCursor::new(&tokens, 3);
        assert_eq!(cursor.peek_kind(), Some(&CssTokenKind::Ident("a".into())));
        assert!(cursor.advance().is_some()); // ident
        assert!(cursor.advance().is_some()); // whitespace
        assert!(cursor.advance().is_some()); // ident
        assert!(cursor.advance().is_none());
        assert!(cursor.peek().is_none());
    }

    #[test]
    fn whitespace_is_in_the_stream_until_skipped() {
        let tokens = toks("a b");
        let mut cursor = TokenCursor::new(&tokens, 3);
        cursor.advance();
        assert_eq!(cursor.peek_kind(), Some(&CssTokenKind::Whitespace));
        cursor.skip_ws();
        assert_eq!(cursor.peek_kind(), Some(&CssTokenKind::Ident("b".into())));
    }

    #[test]
    fn here_span_reports_an_empty_span_at_eof() {
        let tokens = toks("ab");
        let mut cursor = TokenCursor::new(&tokens, 2);
        assert_eq!(cursor.here_span(), SourceSpan::new(0, 2));
        cursor.advance();
        assert_eq!(cursor.here_span(), SourceSpan::empty(cursor.eof()));
    }

    #[test]
    fn slice_span_covers_first_through_last_and_falls_back_to_eof() {
        let tokens = toks("ab cd");
        assert_eq!(slice_span(&tokens, 5), SourceSpan::new(0, 5));
        assert_eq!(slice_span(&[], 5), SourceSpan::empty(5));
    }
}
