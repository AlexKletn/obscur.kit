//! The [`MediaQueryParser`] and its parsed media-query AST (spec §4.13).
//!
//! Parses an `@media` prelude: qualifiers (`not`/`only`), media types
//! (`screen`, `all`, `print`), `and`/`or` connectors, colon feature queries
//! (`(min-width: 600px)`, `(prefers-color-scheme: dark)`), boolean features
//! (`(hover)`), and range features (`(width >= 600px)`). Feature values reuse
//! the crate's [`ParsedValue`] model.

use crate::error::{CssParseError, CssParseErrorKind};
use crate::source::CssSource;

use super::stylesheet_parser::{CssToken, CssTokenKind, Tokenizer};
use super::value_parser::{ParsedValue, ValueParser};

/// A comma-separated list of media queries.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParsedMediaQueryList(pub Vec<ParsedMediaQuery>);

/// A single media query: an optional qualifier and media type followed by zero
/// or more feature conditions.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParsedMediaQuery {
    /// The `not`/`only` qualifier, if present.
    pub qualifier: Option<MediaQualifier>,
    /// The media type (`screen`, `all`, `print`, …), if present.
    pub media_type: Option<Box<str>>,
    /// The feature conditions, in source order.
    pub conditions: Vec<MediaCondition>,
}

/// A media query qualifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaQualifier {
    /// The `not` qualifier.
    Not,
    /// The `only` qualifier.
    Only,
}

/// A single media feature condition (`(min-width: 600px)` or `(width >= 600px)`).
#[derive(Debug, Clone, PartialEq)]
pub struct MediaCondition {
    /// The feature name (for example `min-width`, `prefers-color-scheme`).
    pub feature: Box<str>,
    /// The range comparator, or [`None`] for a colon or boolean feature.
    pub comparator: Option<MediaComparator>,
    /// The feature value, or [`None`] for a boolean feature such as `(hover)`.
    pub value: Option<Vec<ParsedValue>>,
}

/// A comparator used in range media/container features.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaComparator {
    /// `=` equality.
    Eq,
    /// `<` strictly less than.
    Lt,
    /// `<=` less than or equal.
    Le,
    /// `>` strictly greater than.
    Gt,
    /// `>=` greater than or equal.
    Ge,
}

/// Parses an `@media` prelude into a [`ParsedMediaQueryList`].
///
/// The parser is stateless; a single instance may parse any number of preludes.
#[derive(Debug, Clone, Default)]
pub struct MediaQueryParser {
    _private: (),
}

impl MediaQueryParser {
    /// Creates a media-query parser.
    #[must_use]
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Parses `source` as a media-query prelude (without the `@media` keyword).
    ///
    /// # Errors
    ///
    /// Returns a [`CssParseError`] with kind
    /// [`InvalidAtRule`](CssParseErrorKind::InvalidAtRule) on malformed syntax.
    pub fn parse(&self, source: &CssSource) -> Result<ParsedMediaQueryList, CssParseError> {
        let text = source.text();
        let eof = u32::try_from(text.len()).unwrap_or(u32::MAX);
        let (tokens, mut errors) = Tokenizer::new(text).tokenize();
        if !errors.is_empty() {
            return Err(errors.remove(0));
        }
        Self::parse_tokens(&tokens, eof)
    }

    /// Parses a media-query prelude from an already-tokenized slice.
    ///
    /// # Errors
    ///
    /// Returns a [`CssParseError`] on malformed syntax.
    pub(crate) fn parse_tokens(
        tokens: &[CssToken],
        eof: u32,
    ) -> Result<ParsedMediaQueryList, CssParseError> {
        let mut inner = Inner {
            tokens,
            pos: 0,
            eof,
        };
        inner.parse_list()
    }
}

/// Index-based feature/query parser shared in shape with the container parser.
pub(crate) struct Inner<'t> {
    tokens: &'t [CssToken],
    pos: usize,
    eof: u32,
}

impl<'t> Inner<'t> {
    pub(crate) fn new(tokens: &'t [CssToken], eof: u32) -> Self {
        Self {
            tokens,
            pos: 0,
            eof,
        }
    }

    fn kind(&self) -> Option<&'t CssTokenKind> {
        self.tokens.get(self.pos).map(CssToken::kind)
    }

    fn skip_ws(&mut self) {
        while matches!(self.tokens.get(self.pos), Some(t) if t.is_whitespace()) {
            self.pos += 1;
        }
    }

    fn err(&self, message: &str) -> CssParseError {
        let span = self.tokens.get(self.pos).map_or_else(
            || crate::source::SourceSpan::empty(self.eof),
            CssToken::span,
        );
        CssParseError::new(CssParseErrorKind::InvalidAtRule, message, span)
    }

    fn parse_list(&mut self) -> Result<ParsedMediaQueryList, CssParseError> {
        let mut queries = Vec::new();
        loop {
            let query = self.parse_query()?;
            queries.push(query);
            self.skip_ws();
            match self.kind() {
                Some(CssTokenKind::Comma) => self.pos += 1,
                None => break,
                Some(_) => return Err(self.err("unexpected token in media query")),
            }
        }
        Ok(ParsedMediaQueryList(queries))
    }

    fn parse_query(&mut self) -> Result<ParsedMediaQuery, CssParseError> {
        self.skip_ws();
        let qualifier = self.take_qualifier();
        self.skip_ws();
        let media_type = self.take_media_type();
        let conditions = self.parse_conditions()?;
        Ok(ParsedMediaQuery {
            qualifier,
            media_type,
            conditions,
        })
    }

    fn take_qualifier(&mut self) -> Option<MediaQualifier> {
        if let Some(CssTokenKind::Ident(id)) = self.kind() {
            let q = if id.eq_ignore_ascii_case("not") {
                Some(MediaQualifier::Not)
            } else if id.eq_ignore_ascii_case("only") {
                Some(MediaQualifier::Only)
            } else {
                None
            };
            if q.is_some() {
                self.pos += 1;
            }
            return q;
        }
        None
    }

    fn take_media_type(&mut self) -> Option<Box<str>> {
        self.take_leading_ident()
    }

    /// Consumes a leading non-connector identifier (a media type or container
    /// name), returning it if present. Shared with the container-query parser.
    pub(crate) fn take_container_name(&mut self) -> Option<Box<str>> {
        self.skip_ws();
        self.take_leading_ident()
    }

    fn take_leading_ident(&mut self) -> Option<Box<str>> {
        if let Some(CssTokenKind::Ident(id)) = self.kind()
            && !is_connector(id)
        {
            let name = id.clone();
            self.pos += 1;
            return Some(name);
        }
        None
    }

    /// Parses zero or more `(feature …)` conditions joined by `and`/`or`.
    pub(crate) fn parse_conditions(&mut self) -> Result<Vec<MediaCondition>, CssParseError> {
        let mut conditions = Vec::new();
        loop {
            self.skip_ws();
            match self.kind() {
                Some(CssTokenKind::Ident(id)) if is_connector(id) => {
                    self.pos += 1; // consume `and`/`or`/`not`
                }
                Some(CssTokenKind::LParen) => conditions.push(self.parse_feature()?),
                _ => break,
            }
        }
        Ok(conditions)
    }

    fn parse_feature(&mut self) -> Result<MediaCondition, CssParseError> {
        self.pos += 1; // consume '('
        self.skip_ws();
        let feature = match self.kind() {
            Some(CssTokenKind::Ident(id)) => {
                let name = id.clone();
                self.pos += 1;
                name
            }
            _ => return Err(self.err("expected a feature name")),
        };
        self.skip_ws();
        match self.kind() {
            Some(CssTokenKind::RParen) => {
                self.pos += 1;
                Ok(MediaCondition {
                    feature,
                    comparator: None,
                    value: None,
                })
            }
            Some(CssTokenKind::Colon) => {
                self.pos += 1;
                let value = self.parse_value_until_rparen()?;
                Ok(MediaCondition {
                    feature,
                    comparator: None,
                    value: Some(value),
                })
            }
            Some(CssTokenKind::Delim('<' | '>' | '=')) => {
                let comparator = self.take_comparator();
                let value = self.parse_value_until_rparen()?;
                Ok(MediaCondition {
                    feature,
                    comparator,
                    value: Some(value),
                })
            }
            _ => Err(self.err("expected ':', a comparator, or ')'")),
        }
    }

    fn take_comparator(&mut self) -> Option<MediaComparator> {
        let first = match self.kind() {
            Some(CssTokenKind::Delim(c @ ('<' | '>' | '='))) => *c,
            _ => return None,
        };
        self.pos += 1;
        let has_eq = matches!(self.kind(), Some(CssTokenKind::Delim('=')));
        let cmp = match first {
            '=' => MediaComparator::Eq,
            '<' if has_eq => MediaComparator::Le,
            '<' => MediaComparator::Lt,
            '>' if has_eq => MediaComparator::Ge,
            _ => MediaComparator::Gt,
        };
        if has_eq && first != '=' {
            self.pos += 1;
        }
        Some(cmp)
    }

    /// Collects tokens up to the matching `)` and parses them as a value.
    fn parse_value_until_rparen(&mut self) -> Result<Vec<ParsedValue>, CssParseError> {
        let start = self.pos;
        let mut depth = 1i32;
        while let Some(kind) = self.kind() {
            match kind {
                CssTokenKind::LParen => depth += 1,
                CssTokenKind::RParen => {
                    depth -= 1;
                    if depth == 0 {
                        let value_tokens = &self.tokens[start..self.pos];
                        self.pos += 1; // consume ')'
                        return ValueParser::parse_tokens(value_tokens, self.eof);
                    }
                }
                _ => {}
            }
            self.pos += 1;
        }
        Err(self.err("unterminated feature query"))
    }
}

/// Returns whether an identifier is a media-condition connector keyword.
fn is_connector(id: &str) -> bool {
    id.eq_ignore_ascii_case("and")
        || id.eq_ignore_ascii_case("or")
        || id.eq_ignore_ascii_case("not")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Result<ParsedMediaQueryList, CssParseError> {
        let src = CssSource::anonymous(text);
        MediaQueryParser::new().parse(&src)
    }

    fn one(text: &str) -> ParsedMediaQuery {
        parse(text).unwrap().0.into_iter().next().unwrap()
    }

    #[test]
    fn parses_media_type() {
        let q = one("screen");
        assert_eq!(q.media_type.as_deref(), Some("screen"));
        assert!(q.qualifier.is_none());
        assert!(q.conditions.is_empty());
    }

    #[test]
    fn parses_qualifier_and_type() {
        let q = one("only screen");
        assert_eq!(q.qualifier, Some(MediaQualifier::Only));
        assert_eq!(q.media_type.as_deref(), Some("screen"));
    }

    #[test]
    fn parses_not_qualifier() {
        let q = one("not print");
        assert_eq!(q.qualifier, Some(MediaQualifier::Not));
        assert_eq!(q.media_type.as_deref(), Some("print"));
    }

    #[test]
    fn parses_min_width_feature() {
        let q = one("screen and (min-width: 600px)");
        assert_eq!(q.media_type.as_deref(), Some("screen"));
        assert_eq!(q.conditions.len(), 1);
        assert_eq!(&*q.conditions[0].feature, "min-width");
        assert_eq!(
            q.conditions[0].value,
            Some(vec![ParsedValue::Dimension {
                value: 600.0,
                unit: "px".into()
            }])
        );
    }

    #[test]
    fn parses_prefers_color_scheme() {
        let q = one("(prefers-color-scheme: dark)");
        assert!(q.media_type.is_none());
        assert_eq!(&*q.conditions[0].feature, "prefers-color-scheme");
        assert_eq!(
            q.conditions[0].value,
            Some(vec![ParsedValue::Keyword("dark".into())])
        );
    }

    #[test]
    fn parses_boolean_feature() {
        let q = one("(hover)");
        assert_eq!(&*q.conditions[0].feature, "hover");
        assert!(q.conditions[0].value.is_none());
        assert!(q.conditions[0].comparator.is_none());
    }

    #[test]
    fn parses_range_feature() {
        let q = one("(width >= 600px)");
        assert_eq!(&*q.conditions[0].feature, "width");
        assert_eq!(q.conditions[0].comparator, Some(MediaComparator::Ge));
        assert_eq!(
            q.conditions[0].value,
            Some(vec![ParsedValue::Dimension {
                value: 600.0,
                unit: "px".into()
            }])
        );
    }

    #[test]
    fn parses_range_less_than() {
        let q = one("(width < 400px)");
        assert_eq!(q.conditions[0].comparator, Some(MediaComparator::Lt));
    }

    #[test]
    fn parses_multiple_conditions() {
        let q = one("screen and (min-width: 600px) and (max-width: 900px)");
        assert_eq!(q.conditions.len(), 2);
    }

    #[test]
    fn parses_query_list() {
        let list = parse("screen, print").unwrap();
        assert_eq!(list.0.len(), 2);
    }

    #[test]
    fn missing_feature_name_errors() {
        assert_eq!(
            parse("(: 600px)").unwrap_err().kind(),
            CssParseErrorKind::InvalidAtRule
        );
    }

    #[test]
    fn unterminated_feature_errors() {
        assert_eq!(
            parse("(min-width: 600px").unwrap_err().kind(),
            CssParseErrorKind::InvalidAtRule
        );
    }
}
