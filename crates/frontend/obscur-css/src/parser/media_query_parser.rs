#![doc = include_str!("media_query_parser.doc/_module.md")]

use crate::error::{CssParseError, CssParseErrorKind};
use crate::source::CssSource;

use super::css_tokenizer::{CssToken, CssTokenKind, Tokenizer};
use super::value_parser::{ParsedValue, ValueParser};

#[doc = include_str!("media_query_parser.doc/ParsedMediaQueryList.md")]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParsedMediaQueryList(pub Vec<ParsedMediaQuery>);

#[doc = include_str!("media_query_parser.doc/ParsedMediaQuery.md")]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParsedMediaQuery {
    #[doc = include_str!("media_query_parser.doc/qualifier.md")]
    pub qualifier: Option<MediaQualifier>,
    #[doc = include_str!("media_query_parser.doc/media_type.md")]
    pub media_type: Option<Box<str>>,
    #[doc = include_str!("media_query_parser.doc/conditions.md")]
    pub conditions: Vec<MediaCondition>,
}

#[doc = include_str!("media_query_parser.doc/MediaQualifier.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaQualifier {
    #[doc = include_str!("media_query_parser.doc/Not.md")]
    Not,
    #[doc = include_str!("media_query_parser.doc/Only.md")]
    Only,
}

#[doc = include_str!("media_query_parser.doc/MediaCondition.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct MediaCondition {
    #[doc = include_str!("media_query_parser.doc/feature.md")]
    pub feature: Box<str>,
    #[doc = include_str!("media_query_parser.doc/comparator.md")]
    pub comparator: Option<MediaComparator>,
    #[doc = include_str!("media_query_parser.doc/value.md")]
    pub value: Option<Vec<ParsedValue>>,
}

#[doc = include_str!("media_query_parser.doc/MediaComparator.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaComparator {
    #[doc = include_str!("media_query_parser.doc/Eq.md")]
    Eq,
    #[doc = include_str!("media_query_parser.doc/Lt.md")]
    Lt,
    #[doc = include_str!("media_query_parser.doc/Le.md")]
    Le,
    #[doc = include_str!("media_query_parser.doc/Gt.md")]
    Gt,
    #[doc = include_str!("media_query_parser.doc/Ge.md")]
    Ge,
}

#[doc = include_str!("media_query_parser.doc/MediaQueryParser.md")]
#[derive(Debug, Clone, Default)]
pub struct MediaQueryParser {
    _private: (),
}

impl MediaQueryParser {
    #[doc = include_str!("media_query_parser.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self { _private: () }
    }

    #[doc = include_str!("media_query_parser.doc/parse.md")]
    pub fn parse(&self, source: &CssSource) -> Result<ParsedMediaQueryList, CssParseError> {
        let text = source.text();
        let eof = u32::try_from(text.len()).unwrap_or(u32::MAX);
        let (tokens, mut errors) = Tokenizer::new(text).tokenize();
        if !errors.is_empty() {
            return Err(errors.remove(0));
        }
        Self::parse_tokens(&tokens, eof)
    }

    #[doc = include_str!("media_query_parser.doc/parse_tokens.md")]
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

#[doc = include_str!("media_query_parser.doc/Inner.md")]
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

    #[doc = include_str!("media_query_parser.doc/take_container_name.md")]
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

    #[doc = include_str!("media_query_parser.doc/parse_conditions.md")]
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

    #[doc = include_str!("media_query_parser.doc/parse_value_until_rparen.md")]
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

#[doc = include_str!("media_query_parser.doc/is_connector.md")]
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
