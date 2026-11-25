#![doc = include_str!("../../docs/api/parser/css_tokenizer.doc/_module.md")]

use crate::error::{CssParseError, CssParseErrorKind};
use crate::source::SourceSpan;

use super::{css_token::CssToken, css_token_kind::CssTokenKind};

/// Returns whether `c` may start a CSS identifier.
fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_' || (c as u32) >= 0x80
}

/// Returns whether `c` may continue a CSS identifier.
fn is_ident_continue(c: char) -> bool {
    is_ident_start(c) || c.is_ascii_digit() || c == '-'
}

#[doc = include_str!("../../docs/api/parser/css_tokenizer.doc/Tokenizer.md")]
pub(crate) struct Tokenizer<'a> {
    text: &'a str,
    chars: Vec<(usize, char)>,
    pos: usize,
    errors: Vec<CssParseError>,
}

impl<'a> Tokenizer<'a> {
    /// Creates a tokenizer over `text`.
    pub(crate) fn new(text: &'a str) -> Self {
        Self {
            text,
            chars: text.char_indices().collect(),
            pos: 0,
            errors: Vec::new(),
        }
    }

    /// Consumes the tokenizer, returning the token stream and any errors
    /// collected (unterminated strings or comments).
    pub(crate) fn tokenize(mut self) -> (Vec<CssToken>, Vec<CssParseError>) {
        let mut tokens = Vec::new();
        while self.pos < self.chars.len() {
            if let Some(tok) = self.next_token() {
                tokens.push(tok);
            }
        }
        (tokens, self.errors)
    }

    /// Returns the char at the current position.
    fn cur(&self) -> Option<char> {
        self.chars.get(self.pos).map(|&(_, c)| c)
    }

    /// Returns the char `k` positions ahead of the current position.
    fn peek(&self, k: usize) -> Option<char> {
        self.chars.get(self.pos + k).map(|&(_, c)| c)
    }

    /// Returns the byte offset for char index `idx` (or the text length at end).
    fn byte(&self, idx: usize) -> u32 {
        let b = self.chars.get(idx).map_or(self.text.len(), |&(b, _)| b);
        u32::try_from(b).unwrap_or(u32::MAX)
    }

    /// Returns the substring covered by char indices `[start, end)`.
    fn slice(&self, start: usize, end: usize) -> &str {
        let a = self.chars.get(start).map_or(self.text.len(), |&(b, _)| b);
        let b = self.chars.get(end).map_or(self.text.len(), |&(b, _)| b);
        &self.text[a..b]
    }

    /// Builds a span covering char indices `[start, end)`.
    fn span(&self, start: usize, end: usize) -> SourceSpan {
        SourceSpan::new(self.byte(start), self.byte(end))
    }

    /// Scans the next token, or [`None`] for skipped input (comments).
    fn next_token(&mut self) -> Option<CssToken> {
        let start = self.pos;
        let c = self.cur()?;

        if c.is_whitespace() {
            while matches!(self.cur(), Some(ch) if ch.is_whitespace()) {
                self.pos += 1;
            }
            return Some(self.emit(CssTokenKind::Whitespace, start));
        }
        if c == '/' && self.peek(1) == Some('*') {
            self.skip_comment(start);
            return None;
        }
        if c == '"' || c == '\'' {
            return Some(self.scan_string(c, start));
        }
        if self.is_number_start() {
            let kind = self.scan_number();
            return Some(self.emit(kind, start));
        }
        if self.is_ident_start_here() {
            let name = self.scan_ident();
            return Some(self.emit(CssTokenKind::Ident(name), start));
        }
        if c == '@' {
            self.pos += 1;
            if self.is_ident_start_here() {
                let name = self.scan_ident();
                return Some(self.emit(CssTokenKind::AtKeyword(name), start));
            }
            return Some(self.emit(CssTokenKind::Delim('@'), start));
        }
        if c == '#' {
            self.pos += 1;
            if matches!(self.cur(), Some(ch) if is_ident_continue(ch)) {
                let name = self.scan_name();
                return Some(self.emit(CssTokenKind::Hash(name), start));
            }
            return Some(self.emit(CssTokenKind::Delim('#'), start));
        }

        self.pos += 1;
        let kind = match c {
            ':' => CssTokenKind::Colon,
            ';' => CssTokenKind::Semicolon,
            ',' => CssTokenKind::Comma,
            '{' => CssTokenKind::LBrace,
            '}' => CssTokenKind::RBrace,
            '(' => CssTokenKind::LParen,
            ')' => CssTokenKind::RParen,
            '[' => CssTokenKind::LBracket,
            ']' => CssTokenKind::RBracket,
            other => CssTokenKind::Delim(other),
        };
        Some(self.emit(kind, start))
    }

    /// Emits a token of `kind` spanning from char index `start` to the current
    /// position.
    fn emit(&self, kind: CssTokenKind, start: usize) -> CssToken {
        CssToken::new(kind, self.span(start, self.pos))
    }

    /// Skips a `/* … */` comment; records an error if it is unterminated.
    fn skip_comment(&mut self, start: usize) {
        self.pos += 2;
        while self.pos < self.chars.len() {
            if self.cur() == Some('*') && self.peek(1) == Some('/') {
                self.pos += 2;
                return;
            }
            self.pos += 1;
        }
        self.errors.push(CssParseError::new(
            CssParseErrorKind::UnexpectedEof,
            "unterminated comment",
            self.span(start, self.pos),
        ));
    }

    /// Scans a quoted string opened by `quote`; records an error and runs to
    /// end-of-input if unterminated.
    fn scan_string(&mut self, quote: char, start: usize) -> CssToken {
        self.pos += 1;
        let content_start = self.pos;
        while let Some(c) = self.cur() {
            if c == '\\' {
                self.pos += 2;
                continue;
            }
            if c == quote {
                let content: Box<str> = self.slice(content_start, self.pos).into();
                self.pos += 1;
                return self.emit(CssTokenKind::Str(content), start);
            }
            self.pos += 1;
        }
        // Unterminated: emit the run to end-of-input and record an error.
        let content: Box<str> = self.slice(content_start, self.pos).into();
        self.errors.push(CssParseError::new(
            CssParseErrorKind::UnterminatedString,
            "unterminated string literal",
            self.span(start, self.pos),
        ));
        self.emit(CssTokenKind::Str(content), start)
    }

    /// Returns whether an identifier starts at the current position (including
    /// leading hyphens for custom properties and vendor prefixes).
    fn is_ident_start_here(&self) -> bool {
        match self.cur() {
            Some(c) if is_ident_start(c) => true,
            Some('-') => matches!(self.peek(1), Some(d) if is_ident_start(d) || d == '-'),
            _ => false,
        }
    }

    /// Returns whether a number starts at the current position.
    fn is_number_start(&self) -> bool {
        match self.cur() {
            Some(c) if c.is_ascii_digit() => true,
            Some('.') => matches!(self.peek(1), Some(d) if d.is_ascii_digit()),
            Some('+' | '-') => match self.peek(1) {
                Some(d) if d.is_ascii_digit() => true,
                Some('.') => matches!(self.peek(2), Some(d) if d.is_ascii_digit()),
                _ => false,
            },
            _ => false,
        }
    }

    /// Scans an identifier (already known to start here) into owned text.
    fn scan_ident(&mut self) -> Box<str> {
        let start = self.pos;
        self.pos += 1;
        while matches!(self.cur(), Some(c) if is_ident_continue(c)) {
            self.pos += 1;
        }
        self.slice(start, self.pos).into()
    }

    /// Scans a hash/name run (letters, digits, `-`, `_`) into owned text.
    fn scan_name(&mut self) -> Box<str> {
        let start = self.pos;
        while matches!(self.cur(), Some(c) if is_ident_continue(c)) {
            self.pos += 1;
        }
        self.slice(start, self.pos).into()
    }

    /// Scans a number, returning [`Number`](CssTokenKind::Number),
    /// [`Percentage`](CssTokenKind::Percentage), or
    /// [`Dimension`](CssTokenKind::Dimension) depending on the trailing unit.
    fn scan_number(&mut self) -> CssTokenKind {
        let start = self.pos;
        if matches!(self.cur(), Some('+' | '-')) {
            self.pos += 1;
        }
        while matches!(self.cur(), Some(c) if c.is_ascii_digit()) {
            self.pos += 1;
        }
        if self.cur() == Some('.') && matches!(self.peek(1), Some(c) if c.is_ascii_digit()) {
            self.pos += 1;
            while matches!(self.cur(), Some(c) if c.is_ascii_digit()) {
                self.pos += 1;
            }
        }
        if matches!(self.cur(), Some('e' | 'E')) {
            let sign = usize::from(matches!(self.peek(1), Some('+' | '-')));
            if matches!(self.peek(1 + sign), Some(c) if c.is_ascii_digit()) {
                self.pos += 1 + sign;
                while matches!(self.cur(), Some(c) if c.is_ascii_digit()) {
                    self.pos += 1;
                }
            }
        }
        let value = self.slice(start, self.pos).parse::<f64>().unwrap_or(0.0);
        if self.cur() == Some('%') {
            self.pos += 1;
            return CssTokenKind::Percentage(value);
        }
        if matches!(self.cur(), Some(c) if is_ident_start(c)) {
            let unit_start = self.pos;
            self.pos += 1;
            while matches!(self.cur(), Some(c) if is_ident_continue(c)) {
                self.pos += 1;
            }
            let unit: Box<str> = self.slice(unit_start, self.pos).into();
            return CssTokenKind::Dimension { value, unit };
        }
        CssTokenKind::Number(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toks(text: &str) -> Vec<CssToken> {
        Tokenizer::new(text).tokenize().0
    }

    fn kinds(text: &str) -> Vec<CssTokenKind> {
        toks(text).into_iter().map(|t| t.kind().clone()).collect()
    }

    #[test]
    fn tokenizes_identifiers_and_custom_props() {
        assert_eq!(kinds("flex"), vec![CssTokenKind::Ident("flex".into())]);
        assert_eq!(
            kinds("--main-color"),
            vec![CssTokenKind::Ident("--main-color".into())]
        );
        assert_eq!(
            kinds("-webkit-box"),
            vec![CssTokenKind::Ident("-webkit-box".into())]
        );
    }

    #[test]
    fn tokenizes_at_keyword_and_hash() {
        assert_eq!(
            kinds("@media"),
            vec![CssTokenKind::AtKeyword("media".into())]
        );
        assert_eq!(kinds("#id"), vec![CssTokenKind::Hash("id".into())]);
        assert_eq!(kinds("#ff0000"), vec![CssTokenKind::Hash("ff0000".into())]);
    }

    #[test]
    fn tokenizes_numbers_percentages_dimensions() {
        assert_eq!(kinds("12"), vec![CssTokenKind::Number(12.0)]);
        assert_eq!(kinds("-3.5"), vec![CssTokenKind::Number(-3.5)]);
        assert_eq!(kinds("50%"), vec![CssTokenKind::Percentage(50.0)]);
        assert_eq!(
            kinds("12px"),
            vec![CssTokenKind::Dimension {
                value: 12.0,
                unit: "px".into()
            }]
        );
        assert_eq!(
            kinds("1.5em"),
            vec![CssTokenKind::Dimension {
                value: 1.5,
                unit: "em".into()
            }]
        );
    }

    #[test]
    fn dot_before_digit_is_number() {
        assert_eq!(kinds(".5"), vec![CssTokenKind::Number(0.5)]);
        assert_eq!(
            kinds(".foo"),
            vec![CssTokenKind::Delim('.'), CssTokenKind::Ident("foo".into())]
        );
    }

    #[test]
    fn scientific_notation_vs_unit() {
        assert_eq!(kinds("1e3"), vec![CssTokenKind::Number(1000.0)]);
        assert_eq!(
            kinds("1em"),
            vec![CssTokenKind::Dimension {
                value: 1.0,
                unit: "em".into()
            }]
        );
    }

    #[test]
    fn tokenizes_strings_both_quotes() {
        assert_eq!(kinds("\"hi\""), vec![CssTokenKind::Str("hi".into())]);
        assert_eq!(kinds("'yo'"), vec![CssTokenKind::Str("yo".into())]);
    }

    #[test]
    fn unterminated_string_records_error() {
        let (tokens, errors) = Tokenizer::new("\"oops").tokenize();
        assert_eq!(tokens.len(), 1);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind(), CssParseErrorKind::UnterminatedString);
    }

    #[test]
    fn comments_are_skipped() {
        assert_eq!(
            kinds("a/* c */b"),
            vec![
                CssTokenKind::Ident("a".into()),
                CssTokenKind::Ident("b".into())
            ]
        );
    }

    #[test]
    fn unterminated_comment_records_error() {
        let (_, errors) = Tokenizer::new("a /* unclosed").tokenize();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind(), CssParseErrorKind::UnexpectedEof);
    }

    #[test]
    fn punctuation_and_delims() {
        assert_eq!(
            kinds("{}();:,>+~"),
            vec![
                CssTokenKind::LBrace,
                CssTokenKind::RBrace,
                CssTokenKind::LParen,
                CssTokenKind::RParen,
                CssTokenKind::Semicolon,
                CssTokenKind::Colon,
                CssTokenKind::Comma,
                CssTokenKind::Delim('>'),
                CssTokenKind::Delim('+'),
                CssTokenKind::Delim('~'),
            ]
        );
    }

    #[test]
    fn spans_are_byte_accurate() {
        let tokens = toks("ab cd");
        assert_eq!(tokens[0].span(), SourceSpan::new(0, 2));
        assert!(tokens[1].is_whitespace());
        assert_eq!(tokens[2].span(), SourceSpan::new(3, 5));
    }

    #[test]
    fn multibyte_is_char_boundary_safe() {
        let tokens = toks("\"é\" a");
        assert_eq!(tokens[0].kind(), &CssTokenKind::Str("é".into()));
        let last = tokens.last().unwrap();
        assert_eq!(last.kind(), &CssTokenKind::Ident("a".into()));
    }
}
