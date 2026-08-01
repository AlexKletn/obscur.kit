#![doc = include_str!("lexer.doc/_module.md")]

use crate::lexer::{Token, TokenKind};

#[doc = include_str!("lexer.doc/Lexer.md")]
#[derive(Debug, Clone, Default)]
pub struct Lexer {
    _private: (),
}

impl Lexer {
    #[doc = include_str!("lexer.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self { _private: () }
    }

    #[doc = include_str!("lexer.doc/tokenize.md")]
    #[must_use]
    pub fn tokenize(&self, source: &str) -> Vec<Token> {
        let chars: Vec<(usize, char)> = source.char_indices().collect();
        let n = chars.len();
        let mut tokens = Vec::new();
        let mut i = 0;

        while i < n {
            let (start_byte, c) = chars[i];
            let (kind, next_i) = scan_token(&chars, i, c);
            let end_byte = if next_i < n {
                chars[next_i].0
            } else {
                source.len()
            };
            let offset = u32::try_from(start_byte).unwrap_or(u32::MAX);
            tokens.push(Token::new(kind, &source[start_byte..end_byte], offset));
            i = next_i;
        }

        let eof_offset = u32::try_from(source.len()).unwrap_or(u32::MAX);
        tokens.push(Token::new(TokenKind::Eof, "", eof_offset));
        tokens
    }
}

#[doc = include_str!("lexer.doc/scan_token.md")]
fn scan_token(chars: &[(usize, char)], i: usize, c: char) -> (TokenKind, usize) {
    let n = chars.len();
    match c {
        '<' => (TokenKind::LAngle, i + 1),
        '>' => (TokenKind::RAngle, i + 1),
        '/' => (TokenKind::Slash, i + 1),
        '=' => (TokenKind::Equals, i + 1),
        '{' => (TokenKind::LBrace, i + 1),
        '}' => (TokenKind::RBrace, i + 1),
        ':' => (TokenKind::Colon, i + 1),
        '"' => (TokenKind::String, scan_string(chars, i)),
        _ if is_ident_start(c) => {
            let mut j = i + 1;
            while j < n && is_ident_continue(chars[j].1) {
                j += 1;
            }
            (TokenKind::Ident, j)
        }
        _ if c.is_ascii_digit() => (TokenKind::Number, scan_number(chars, i)),
        _ if c.is_whitespace() => {
            let mut j = i + 1;
            while j < n && chars[j].1.is_whitespace() {
                j += 1;
            }
            (TokenKind::Whitespace, j)
        }
        _ if is_text_char(c) => {
            let mut j = i + 1;
            while j < n && is_text_char(chars[j].1) {
                j += 1;
            }
            (TokenKind::Text, j)
        }
        _ => (TokenKind::Error, i + 1),
    }
}

#[doc = include_str!("lexer.doc/scan_string.md")]
fn scan_string(chars: &[(usize, char)], i: usize) -> usize {
    let n = chars.len();
    let mut j = i + 1;
    while j < n {
        match chars[j].1 {
            '\\' => j += 2,      // skip the backslash and the escaped char
            '"' => return j + 1, // consume the closing quote
            _ => j += 1,
        }
    }
    n // unterminated: run to end of input
}

#[doc = include_str!("lexer.doc/scan_number.md")]
fn scan_number(chars: &[(usize, char)], i: usize) -> usize {
    let n = chars.len();
    let mut j = i + 1;
    while j < n && chars[j].1.is_ascii_digit() {
        j += 1;
    }
    if j + 1 < n && chars[j].1 == '.' && chars[j + 1].1.is_ascii_digit() {
        j += 2;
        while j < n && chars[j].1.is_ascii_digit() {
            j += 1;
        }
    }
    j
}

#[doc = include_str!("lexer.doc/is_ident_start.md")]
const fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

#[doc = include_str!("lexer.doc/is_ident_continue.md")]
const fn is_ident_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-')
}

#[doc = include_str!("lexer.doc/is_punct.md")]
const fn is_punct(c: char) -> bool {
    matches!(c, '<' | '>' | '/' | '=' | '{' | '}' | ':')
}

#[doc = include_str!("lexer.doc/is_text_char.md")]
fn is_text_char(c: char) -> bool {
    !c.is_whitespace()
        && !is_ident_start(c)
        && !c.is_ascii_digit()
        && !is_punct(c)
        && c != '"'
        && !c.is_control()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Concatenation of every non-`Eof` token text must equal the source.
    fn assert_lossless(source: &str, tokens: &[Token]) {
        let rebuilt: String = tokens
            .iter()
            .filter(|t| t.kind() != TokenKind::Eof)
            .map(Token::text)
            .collect();
        assert_eq!(rebuilt, source);
    }

    #[test]
    fn lexes_a_full_element_losslessly() {
        let src = "<Button label=\"Save\" on:click=submit>{count}</Button>";
        let tokens = Lexer::new().tokenize(src);

        assert_lossless(src, &tokens);
        assert_eq!(tokens.last().unwrap().kind(), TokenKind::Eof);

        // Spot-check the expected token sequence.
        let non_ws: Vec<(TokenKind, &str)> = tokens
            .iter()
            .filter(|t| t.kind() != TokenKind::Whitespace && t.kind() != TokenKind::Eof)
            .map(|t| (t.kind(), t.text()))
            .collect();
        assert_eq!(
            non_ws,
            vec![
                (TokenKind::LAngle, "<"),
                (TokenKind::Ident, "Button"),
                (TokenKind::Ident, "label"),
                (TokenKind::Equals, "="),
                (TokenKind::String, "\"Save\""),
                (TokenKind::Ident, "on"),
                (TokenKind::Colon, ":"),
                (TokenKind::Ident, "click"),
                (TokenKind::Equals, "="),
                (TokenKind::Ident, "submit"),
                (TokenKind::RAngle, ">"),
                (TokenKind::LBrace, "{"),
                (TokenKind::Ident, "count"),
                (TokenKind::RBrace, "}"),
                (TokenKind::LAngle, "<"),
                (TokenKind::Slash, "/"),
                (TokenKind::Ident, "Button"),
                (TokenKind::RAngle, ">"),
            ]
        );
    }

    #[test]
    fn multibyte_utf8_round_trips() {
        let src = "<Δ title=\"café ☕\">Ключ</Δ>";
        let tokens = Lexer::new().tokenize(src);
        assert_lossless(src, &tokens);
        assert_eq!(tokens.last().unwrap().kind(), TokenKind::Eof);
    }

    #[test]
    fn escaped_quote_does_not_terminate_string() {
        let src = "\"a\\\"b\"";
        let tokens = Lexer::new().tokenize(src);
        assert_eq!(tokens[0].kind(), TokenKind::String);
        assert_eq!(tokens[0].text(), src);
        assert_lossless(src, &tokens);
    }

    #[test]
    fn number_stops_at_trailing_dot() {
        let tokens = Lexer::new().tokenize("3.14");
        assert_eq!(tokens[0].kind(), TokenKind::Number);
        assert_eq!(tokens[0].text(), "3.14");

        let tokens = Lexer::new().tokenize("42.");
        assert_eq!(tokens[0].kind(), TokenKind::Number);
        assert_eq!(tokens[0].text(), "42");
    }

    #[test]
    fn offsets_are_contiguous() {
        let src = "<a>x</a>";
        let tokens = Lexer::new().tokenize(src);
        let mut expected = 0u32;
        for tok in &tokens {
            assert_eq!(tok.offset(), expected);
            expected = tok.end();
        }
        assert_eq!(usize::try_from(expected).unwrap(), src.len());
    }

    #[test]
    fn empty_source_yields_only_eof() {
        let tokens = Lexer::new().tokenize("");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind(), TokenKind::Eof);
        assert_eq!(tokens[0].offset(), 0);
    }
}
