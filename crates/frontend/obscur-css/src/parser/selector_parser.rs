//! The [`SelectorParser`] and its parsed selector AST (spec §4.13).
//!
//! Supports the toolkit's selector surface: type/universal, class, id,
//! attribute (`[a]`, `[a=b]`, `[a^=b]`, `[a$=b]`, `[a*=b]`, `[a~=b]`, `[a|=b]`),
//! pseudo-class (including functional forms whose argument is captured as raw
//! text), pseudo-element (`::before`/`::after`), and the four combinators. No
//! specificity is computed here — that is the compiler's responsibility.

use crate::error::{CssParseError, CssParseErrorKind};
use crate::source::{CssSource, SourceSpan};

use super::stylesheet_parser::{CssToken, CssTokenKind, TokenCursor, Tokenizer, slice_span};

/// A comma-separated list of complex selectors (`a, b > c`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParsedSelectorList(pub Vec<ParsedComplexSelector>);

/// A complex selector: a lead compound selector followed by combinator-linked
/// compounds (`div > .a + span`).
///
/// The `head` is the first compound; each entry in `tail` pairs the combinator
/// that *precedes* a compound with that compound.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedComplexSelector {
    /// The first compound selector.
    pub head: ParsedCompoundSelector,
    /// The remaining compounds, each with its preceding combinator.
    pub tail: Vec<(Combinator, ParsedCompoundSelector)>,
}

/// A combinator linking two compound selectors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combinator {
    /// Descendant combinator (whitespace).
    Descendant,
    /// Child combinator (`>`).
    Child,
    /// Next-sibling combinator (`+`).
    NextSibling,
    /// Subsequent-sibling combinator (`~`).
    SubsequentSibling,
}

/// A compound selector: contiguous simple selectors with no combinator between
/// them (`div.a#b:hover`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParsedCompoundSelector {
    /// The simple selectors making up the compound, in source order.
    pub parts: Vec<SimpleSelector>,
}

/// A single simple selector.
#[derive(Debug, Clone, PartialEq)]
pub enum SimpleSelector {
    /// A type/element selector such as `div`.
    Type(Box<str>),
    /// The universal selector `*`.
    Universal,
    /// A class selector such as `.button`.
    Class(Box<str>),
    /// An id selector such as `#main`.
    Id(Box<str>),
    /// An attribute selector such as `[data-x]` or `[type="text"]`.
    Attribute {
        /// The attribute name.
        name: Box<str>,
        /// The match operator, or [`None`] for a bare presence test.
        op: Option<AttrOp>,
        /// The compared value, or [`None`] for a bare presence test.
        value: Option<Box<str>>,
    },
    /// A pseudo-class such as `:hover` or `:nth-child(2n+1)`.
    PseudoClass {
        /// The pseudo-class name (without the leading `:`).
        name: Box<str>,
        /// The raw argument text for a functional pseudo-class, if any.
        arg: Option<Box<str>>,
    },
    /// A pseudo-element such as `::before` (without the leading `::`).
    PseudoElement(Box<str>),
}

/// An attribute selector match operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttrOp {
    /// `=` exact match.
    Exact,
    /// `~=` whitespace-separated list contains.
    Includes,
    /// `|=` exact or prefix followed by `-`.
    DashMatch,
    /// `^=` prefix match.
    Prefix,
    /// `$=` suffix match.
    Suffix,
    /// `*=` substring match.
    Substring,
}

/// Parses selector tokens into a [`ParsedSelectorList`].
///
/// The parser is stateless; a single instance may parse any number of selectors.
#[derive(Debug, Clone, Default)]
pub struct SelectorParser {
    _private: (),
}

impl SelectorParser {
    /// Creates a selector parser.
    #[must_use]
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Parses `source` as a selector list.
    ///
    /// # Errors
    ///
    /// Returns a [`CssParseError`] with kind
    /// [`InvalidSelector`](CssParseErrorKind::InvalidSelector) on malformed
    /// input (an empty compound, an unterminated `[ … ]`, and so on).
    pub fn parse(&self, source: &CssSource) -> Result<ParsedSelectorList, CssParseError> {
        let text = source.text();
        let eof = u32::try_from(text.len()).unwrap_or(u32::MAX);
        let (tokens, mut errors) = Tokenizer::new(text).tokenize();
        if !errors.is_empty() {
            return Err(errors.remove(0));
        }
        Self::parse_tokens(&tokens, text, eof)
    }

    /// Parses a selector list from an already-tokenized slice. `source` is the
    /// full source text, used to slice raw functional-pseudo arguments.
    ///
    /// # Errors
    ///
    /// Returns a [`CssParseError`] on malformed selector syntax.
    pub(crate) fn parse_tokens(
        tokens: &[CssToken],
        source: &str,
        eof: u32,
    ) -> Result<ParsedSelectorList, CssParseError> {
        let mut parser = Inner {
            cursor: TokenCursor::new(tokens, eof),
            source,
            span: slice_span(tokens, eof),
        };
        parser.parse_list()
    }
}

/// The inner selector parser state.
struct Inner<'a, 't> {
    cursor: TokenCursor<'t>,
    source: &'a str,
    span: SourceSpan,
}

impl Inner<'_, '_> {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Result<ParsedSelectorList, CssParseError> {
        let src = CssSource::anonymous(text);
        SelectorParser::new().parse(&src)
    }

    fn one(text: &str) -> ParsedComplexSelector {
        parse(text).unwrap().0.into_iter().next().unwrap()
    }

    #[test]
    fn parses_type_selector() {
        let c = one("div");
        assert_eq!(c.head.parts, vec![SimpleSelector::Type("div".into())]);
        assert!(c.tail.is_empty());
    }

    #[test]
    fn parses_universal() {
        assert_eq!(one("*").head.parts, vec![SimpleSelector::Universal]);
    }

    #[test]
    fn parses_class_and_id() {
        assert_eq!(
            one(".btn").head.parts,
            vec![SimpleSelector::Class("btn".into())]
        );
        assert_eq!(
            one("#main").head.parts,
            vec![SimpleSelector::Id("main".into())]
        );
    }

    #[test]
    fn parses_compound() {
        let c = one("div.a#b");
        assert_eq!(
            c.head.parts,
            vec![
                SimpleSelector::Type("div".into()),
                SimpleSelector::Class("a".into()),
                SimpleSelector::Id("b".into()),
            ]
        );
    }

    #[test]
    fn parses_descendant_and_child() {
        let c = one("div .a");
        assert_eq!(c.tail.len(), 1);
        assert_eq!(c.tail[0].0, Combinator::Descendant);

        let c = one("div > .a");
        assert_eq!(c.tail[0].0, Combinator::Child);
    }

    #[test]
    fn parses_sibling_combinators() {
        assert_eq!(one("a + b").tail[0].0, Combinator::NextSibling);
        assert_eq!(one("a ~ b").tail[0].0, Combinator::SubsequentSibling);
    }

    #[test]
    fn parses_selector_list() {
        let list = parse("a, b, c").unwrap();
        assert_eq!(list.0.len(), 3);
    }

    #[test]
    fn parses_attribute_presence() {
        let c = one("[disabled]");
        assert_eq!(
            c.head.parts,
            vec![SimpleSelector::Attribute {
                name: "disabled".into(),
                op: None,
                value: None
            }]
        );
    }

    #[test]
    fn parses_attribute_operators() {
        let cases = [
            ("[a=b]", AttrOp::Exact),
            ("[a~=b]", AttrOp::Includes),
            ("[a|=b]", AttrOp::DashMatch),
            ("[a^=b]", AttrOp::Prefix),
            ("[a$=b]", AttrOp::Suffix),
            ("[a*=b]", AttrOp::Substring),
        ];
        for (text, expected) in cases {
            let c = one(text);
            let SimpleSelector::Attribute { op, .. } = &c.head.parts[0] else {
                panic!("expected attribute selector for {text}");
            };
            assert_eq!(*op, Some(expected), "for {text}");
        }
    }

    #[test]
    fn parses_attribute_quoted_value() {
        let c = one("[type=\"text\"]");
        let SimpleSelector::Attribute { value, .. } = &c.head.parts[0] else {
            panic!("expected attribute selector");
        };
        assert_eq!(value.as_deref(), Some("text"));
    }

    #[test]
    fn parses_pseudo_class() {
        let c = one(":hover");
        assert_eq!(
            c.head.parts,
            vec![SimpleSelector::PseudoClass {
                name: "hover".into(),
                arg: None
            }]
        );
    }

    #[test]
    fn parses_functional_pseudo_class() {
        let c = one(":nth-child(2n + 1)");
        let SimpleSelector::PseudoClass { name, arg } = &c.head.parts[0] else {
            panic!("expected pseudo-class");
        };
        assert_eq!(&**name, "nth-child");
        assert_eq!(arg.as_deref(), Some("2n + 1"));
    }

    #[test]
    fn parses_pseudo_element() {
        let c = one("::before");
        assert_eq!(
            c.head.parts,
            vec![SimpleSelector::PseudoElement("before".into())]
        );
    }

    #[test]
    fn complex_with_pseudo_and_combinator() {
        let c = one("ul > li:hover");
        assert_eq!(c.head.parts, vec![SimpleSelector::Type("ul".into())]);
        assert_eq!(c.tail.len(), 1);
        assert_eq!(c.tail[0].0, Combinator::Child);
        assert_eq!(
            c.tail[0].1.parts,
            vec![
                SimpleSelector::Type("li".into()),
                SimpleSelector::PseudoClass {
                    name: "hover".into(),
                    arg: None
                },
            ]
        );
    }

    #[test]
    fn empty_selector_is_error() {
        assert_eq!(
            parse("").unwrap_err().kind(),
            CssParseErrorKind::InvalidSelector
        );
    }

    #[test]
    fn dangling_class_is_error() {
        assert_eq!(
            parse(".").unwrap_err().kind(),
            CssParseErrorKind::InvalidSelector
        );
    }

    #[test]
    fn unterminated_attribute_is_error() {
        assert_eq!(
            parse("[a").unwrap_err().kind(),
            CssParseErrorKind::InvalidSelector
        );
    }
}
