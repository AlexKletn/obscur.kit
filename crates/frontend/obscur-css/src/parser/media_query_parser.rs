#![doc = include_str!("../../docs/api/parser/media_query_parser.doc/_module.md")]

use crate::error::CssParseError;
use crate::source::CssSource;

use super::query_condition_parser::QueryConditionParser;
use super::{MediaQualifier, ParsedMediaQuery, ParsedMediaQueryList};
use super::{css_token::CssToken, css_token_kind::CssTokenKind, css_tokenizer::Tokenizer};

#[doc = include_str!("../../docs/api/parser/media_query_parser.doc/MediaQueryParser.md")]
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

    #[doc = include_str!("../../docs/api/parser/media_query_parser.doc/parse.md")]
    pub fn parse(&self, source: &CssSource) -> Result<ParsedMediaQueryList, CssParseError> {
        let text = source.text();
        let eof = u32::try_from(text.len()).unwrap_or(u32::MAX);
        let (tokens, mut errors) = Tokenizer::new(text).tokenize();
        if !errors.is_empty() {
            return Err(errors.remove(0));
        }
        Self::parse_tokens(&tokens, eof)
    }

    #[doc = include_str!("../../docs/api/parser/media_query_parser.doc/parse_tokens.md")]
    pub(crate) fn parse_tokens(
        tokens: &[CssToken],
        eof: u32,
    ) -> Result<ParsedMediaQueryList, CssParseError> {
        let (list, errors) = Self::parse_tokens_recovering(tokens, eof);
        errors.into_iter().next().map_or(Ok(list), Err)
    }

    /// Parses each top-level branch independently and returns syntax errors alongside the recovered list.
    pub(crate) fn parse_tokens_recovering(
        tokens: &[CssToken],
        eof: u32,
    ) -> (ParsedMediaQueryList, Vec<CssParseError>) {
        if tokens.iter().all(CssToken::is_whitespace) {
            return (ParsedMediaQueryList::default(), Vec::new());
        }
        let mut queries = Vec::new();
        let mut errors = Vec::new();
        let mut start = 0;
        let mut closers = Vec::new();
        for (index, token) in tokens.iter().enumerate() {
            match token.kind() {
                CssTokenKind::LParen => closers.push(CssTokenKind::RParen),
                CssTokenKind::LBracket => closers.push(CssTokenKind::RBracket),
                CssTokenKind::LBrace => closers.push(CssTokenKind::RBrace),
                kind if closers.last() == Some(kind) => {
                    closers.pop();
                }
                CssTokenKind::Comma if closers.is_empty() => {
                    queries.push(parse_media_branch(
                        &tokens[start..index],
                        token.span().start(),
                        &mut errors,
                    ));
                    start = index + 1;
                }
                _ => {}
            }
        }
        queries.push(parse_media_branch(&tokens[start..], eof, &mut errors));
        (ParsedMediaQueryList(queries), errors)
    }
}

/// Parses one complete media query and substitutes not all when its syntax is invalid.
fn parse_media_branch(
    tokens: &[CssToken],
    eof: u32,
    errors: &mut Vec<CssParseError>,
) -> ParsedMediaQuery {
    let mut inner = QueryConditionParser::new(tokens, eof);
    let query = inner.parse_query().and_then(|query| {
        inner.skip_ws();
        if inner.kind().is_some() {
            return Err(inner.err("unexpected token in media query"));
        }
        Ok(query)
    });
    query.unwrap_or_else(|error| {
        errors.push(error);
        ParsedMediaQuery {
            qualifier: Some(MediaQualifier::Not),
            media_type: Some("all".into()),
            ..ParsedMediaQuery::default()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::CssParseErrorKind;
    use crate::parser::{MediaComparator, ParsedValue};

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
