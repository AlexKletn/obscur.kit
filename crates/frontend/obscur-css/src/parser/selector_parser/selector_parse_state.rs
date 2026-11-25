use crate::error::{CssParseError, CssParseErrorKind};
use crate::source::SourceSpan;

use super::super::token_cursor::{TokenCursor, slice_span};
use super::super::{css_token::CssToken, css_token_kind::CssTokenKind};

use super::super::{
    AttrOp, Combinator, ParsedComplexSelector, ParsedCompoundSelector, ParsedSelectorList,
    SimpleSelector,
};

/// The inner selector parser state.
pub(super) struct SelectorParseState<'a, 't> {
    cursor: TokenCursor<'t>,
    source: &'a str,
    span: SourceSpan,
}

impl<'a, 't> SelectorParseState<'a, 't> {
    pub(super) fn parse(
        tokens: &'t [CssToken],
        source: &'a str,
        eof: u32,
    ) -> Result<ParsedSelectorList, CssParseError> {
        let mut parser = Self {
            cursor: TokenCursor::new(tokens, eof),
            source,
            span: slice_span(tokens, eof),
        };
        parser.parse_list()
    }

    fn error(&self, message: &str) -> CssParseError {
        CssParseError::new(CssParseErrorKind::InvalidSelector, message, self.span)
    }

    fn parse_list(&mut self) -> Result<ParsedSelectorList, CssParseError> {
        let mut selectors = Vec::new();
        loop {
            let complex = self.parse_complex()?;
            selectors.push(complex);
            self.cursor.skip_ws();
            match self.cursor.peek_kind() {
                Some(CssTokenKind::Comma) => {
                    self.cursor.advance();
                }
                None => break,
                Some(_) => return Err(self.error("unexpected token after selector")),
            }
        }
        if selectors.is_empty() {
            return Err(self.error("empty selector"));
        }
        Ok(ParsedSelectorList(selectors))
    }

    fn parse_complex(&mut self) -> Result<ParsedComplexSelector, CssParseError> {
        self.cursor.skip_ws();
        let head = self.parse_compound()?;
        let mut tail = Vec::new();
        loop {
            let had_ws = self.skip_ws_reporting();
            let combinator = match self.cursor.peek_kind() {
                None | Some(CssTokenKind::Comma) => break,
                Some(CssTokenKind::Delim('>')) => {
                    self.cursor.advance();
                    Combinator::Child
                }
                Some(CssTokenKind::Delim('+')) => {
                    self.cursor.advance();
                    Combinator::NextSibling
                }
                Some(CssTokenKind::Delim('~')) => {
                    self.cursor.advance();
                    Combinator::SubsequentSibling
                }
                Some(_) if had_ws => Combinator::Descendant,
                Some(_) => return Err(self.error("unexpected token in selector")),
            };
            self.cursor.skip_ws();
            let compound = self.parse_compound()?;
            tail.push((combinator, compound));
        }
        Ok(ParsedComplexSelector { head, tail })
    }

    /// Skips whitespace, returning whether any was skipped.
    fn skip_ws_reporting(&mut self) -> bool {
        let mut had = false;
        while matches!(self.cursor.peek(), Some(t) if t.is_whitespace()) {
            self.cursor.advance();
            had = true;
        }
        had
    }

    fn parse_compound(&mut self) -> Result<ParsedCompoundSelector, CssParseError> {
        let mut parts = Vec::new();
        loop {
            match self.cursor.peek_kind() {
                Some(CssTokenKind::Delim('*')) => {
                    self.cursor.advance();
                    parts.push(SimpleSelector::Universal);
                }
                Some(CssTokenKind::Ident(name)) => {
                    let name = name.clone();
                    self.cursor.advance();
                    parts.push(SimpleSelector::Type(name));
                }
                Some(CssTokenKind::Delim('.')) => {
                    self.cursor.advance();
                    let name = self.expect_ident("expected a class name after '.'")?;
                    parts.push(SimpleSelector::Class(name));
                }
                Some(CssTokenKind::Hash(name)) => {
                    let name = name.clone();
                    self.cursor.advance();
                    parts.push(SimpleSelector::Id(name));
                }
                Some(CssTokenKind::LBracket) => {
                    self.cursor.advance();
                    parts.push(self.parse_attribute()?);
                }
                Some(CssTokenKind::Colon) => {
                    parts.push(self.parse_pseudo()?);
                }
                _ => break,
            }
        }
        if parts.is_empty() {
            return Err(self.error("expected a compound selector"));
        }
        Ok(ParsedCompoundSelector { parts })
    }

    fn parse_attribute(&mut self) -> Result<SimpleSelector, CssParseError> {
        self.cursor.skip_ws();
        let name = self.expect_ident("expected an attribute name")?;
        self.cursor.skip_ws();
        if matches!(self.cursor.peek_kind(), Some(CssTokenKind::RBracket)) {
            self.cursor.advance();
            return Ok(SimpleSelector::Attribute {
                name,
                op: None,
                value: None,
            });
        }
        let op = self.parse_attr_op()?;
        self.cursor.skip_ws();
        let value = match self.cursor.peek_kind() {
            Some(CssTokenKind::Str(s) | CssTokenKind::Ident(s)) => {
                let s = s.clone();
                self.cursor.advance();
                s
            }
            _ => return Err(self.error("expected an attribute value")),
        };
        self.cursor.skip_ws();
        // Tolerate an optional case-sensitivity flag (`i`/`s`).
        if matches!(self.cursor.peek_kind(), Some(CssTokenKind::Ident(_))) {
            self.cursor.advance();
            self.cursor.skip_ws();
        }
        if !matches!(self.cursor.peek_kind(), Some(CssTokenKind::RBracket)) {
            return Err(self.error("expected ']' to close attribute selector"));
        }
        self.cursor.advance();
        Ok(SimpleSelector::Attribute {
            name,
            op: Some(op),
            value: Some(value),
        })
    }

    /// Parses an attribute operator: `=`, or one of `~ | ^ $ *` followed by `=`.
    fn parse_attr_op(&mut self) -> Result<AttrOp, CssParseError> {
        match self.cursor.peek_kind() {
            Some(CssTokenKind::Delim('=')) => {
                self.cursor.advance();
                Ok(AttrOp::Exact)
            }
            Some(CssTokenKind::Delim(prefix @ ('~' | '|' | '^' | '$' | '*'))) => {
                let prefix = *prefix;
                self.cursor.advance();
                if !matches!(self.cursor.peek_kind(), Some(CssTokenKind::Delim('='))) {
                    return Err(self.error("expected '=' in attribute operator"));
                }
                self.cursor.advance();
                Ok(match prefix {
                    '~' => AttrOp::Includes,
                    '|' => AttrOp::DashMatch,
                    '^' => AttrOp::Prefix,
                    '$' => AttrOp::Suffix,
                    _ => AttrOp::Substring,
                })
            }
            _ => Err(self.error("expected an attribute operator")),
        }
    }

    fn parse_pseudo(&mut self) -> Result<SimpleSelector, CssParseError> {
        self.cursor.advance(); // consume first ':'
        // A second colon marks a pseudo-element.
        if matches!(self.cursor.peek_kind(), Some(CssTokenKind::Colon)) {
            self.cursor.advance();
            let name = self.expect_ident("expected a pseudo-element name")?;
            // A functional pseudo-element carries a parenthesised argument.
            // `::part(name)` is the one we model (§2.12 / §4.15); any other
            // `::foo(...)` keeps its bare pseudo-element name but its argument is
            // consumed so the compound parse does not choke on the `(`.
            if matches!(self.cursor.peek_kind(), Some(CssTokenKind::LParen)) {
                let arg = self.capture_paren_arg()?;
                if name.eq_ignore_ascii_case("part") {
                    return Ok(SimpleSelector::Part(arg));
                }
                return Ok(SimpleSelector::PseudoElement(name));
            }
            return Ok(SimpleSelector::PseudoElement(name));
        }
        let name = self.expect_ident("expected a pseudo-class name")?;
        if matches!(self.cursor.peek_kind(), Some(CssTokenKind::LParen)) {
            let arg = self.capture_paren_arg()?;
            return Ok(SimpleSelector::PseudoClass {
                name,
                arg: Some(arg),
            });
        }
        Ok(SimpleSelector::PseudoClass { name, arg: None })
    }

    /// Captures the raw text between a `(` and its matching `)` as an owned
    /// string, consuming both parentheses.
    fn capture_paren_arg(&mut self) -> Result<Box<str>, CssParseError> {
        self.cursor.advance(); // consume '('
        let mut depth = 1i32;
        let mut first: Option<SourceSpan> = None;
        let mut last: Option<SourceSpan> = None;
        while let Some(tok) = self.cursor.peek() {
            match tok.kind() {
                CssTokenKind::LParen => depth += 1,
                CssTokenKind::RParen => {
                    depth -= 1;
                    if depth == 0 {
                        self.cursor.advance();
                        let raw = match (first, last) {
                            (Some(f), Some(l)) => {
                                let span = f.union(l);
                                let a = usize::try_from(span.start()).unwrap_or(0);
                                let b = usize::try_from(span.end()).unwrap_or(0);
                                self.source.get(a..b).unwrap_or("").trim()
                            }
                            _ => "",
                        };
                        return Ok(raw.into());
                    }
                }
                _ => {}
            }
            if !tok.is_whitespace() {
                let span = tok.span();
                if first.is_none() {
                    first = Some(span);
                }
                last = Some(span);
            }
            self.cursor.advance();
        }
        Err(self.error("unterminated functional pseudo-class"))
    }

    fn expect_ident(&mut self, message: &str) -> Result<Box<str>, CssParseError> {
        match self.cursor.peek_kind() {
            Some(CssTokenKind::Ident(name)) => {
                let name = name.clone();
                self.cursor.advance();
                Ok(name)
            }
            _ => Err(self.error(message)),
        }
    }
}
