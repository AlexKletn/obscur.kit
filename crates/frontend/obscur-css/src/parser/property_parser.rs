#![doc = include_str!("property_parser.doc/_module.md")]

use crate::error::{CssParseError, CssParseErrorKind};
use crate::source::{CssSource, SourceSpan};

use super::css_tokenizer::{CssToken, CssTokenKind, Tokenizer};
use super::token_cursor::slice_span;
use super::value_parser::{ParsedValue, ValueParser};

#[doc = include_str!("property_parser.doc/ParsedDeclaration.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedDeclaration {
    #[doc = include_str!("property_parser.doc/name.md")]
    pub name: Box<str>,
    #[doc = include_str!("property_parser.doc/value.md")]
    pub value: Vec<ParsedValue>,
    #[doc = include_str!("property_parser.doc/important.md")]
    pub important: bool,
    #[doc = include_str!("property_parser.doc/span.md")]
    pub span: SourceSpan,
}

#[doc = include_str!("property_parser.doc/PropertyParser.md")]
#[derive(Debug, Clone, Default)]
pub struct PropertyParser {
    _private: (),
}

impl PropertyParser {
    #[doc = include_str!("property_parser.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self { _private: () }
    }

    #[doc = include_str!("property_parser.doc/parse.md")]
    pub fn parse(&self, source: &CssSource) -> Result<ParsedDeclaration, CssParseError> {
        let text = source.text();
        let eof = u32::try_from(text.len()).unwrap_or(u32::MAX);
        let (tokens, mut errors) = Tokenizer::new(text).tokenize();
        if !errors.is_empty() {
            return Err(errors.remove(0));
        }
        Self::parse_tokens(&tokens, eof)
    }

    #[doc = include_str!("property_parser.doc/parse_tokens.md")]
    pub(crate) fn parse_tokens(
        tokens: &[CssToken],
        eof: u32,
    ) -> Result<ParsedDeclaration, CssParseError> {
        let span = slice_span(tokens, eof);
        let mut idx = skip_ws(tokens, 0);
        let name = match tokens.get(idx).map(CssToken::kind) {
            Some(CssTokenKind::Ident(name)) => name.clone(),
            _ => {
                return Err(CssParseError::unexpected_token(
                    "expected a property name",
                    span,
                ));
            }
        };
        idx = skip_ws(tokens, idx + 1);

        if !matches!(
            tokens.get(idx).map(CssToken::kind),
            Some(CssTokenKind::Colon)
        ) {
            return Err(CssParseError::new(
                CssParseErrorKind::ExpectedColon,
                "expected ':' after property name",
                span,
            ));
        }
        idx += 1;

        let value_tokens = trim_ws(&tokens[idx..]);
        let (value_tokens, important) = strip_important(value_tokens);
        let value_tokens = trim_ws(value_tokens);

        if value_tokens.is_empty() {
            return Err(CssParseError::new(
                CssParseErrorKind::InvalidValue,
                "declaration has no value",
                span,
            ));
        }
        let value = ValueParser::parse_tokens(value_tokens, eof)?;
        Ok(ParsedDeclaration {
            name,
            value,
            important,
            span,
        })
    }
}

#[doc = include_str!("property_parser.doc/skip_ws.md")]
fn skip_ws(tokens: &[CssToken], from: usize) -> usize {
    let mut i = from;
    while matches!(tokens.get(i), Some(t) if t.is_whitespace()) {
        i += 1;
    }
    i
}

#[doc = include_str!("property_parser.doc/trim_ws.md")]
fn trim_ws(tokens: &[CssToken]) -> &[CssToken] {
    let mut start = 0;
    let mut end = tokens.len();
    while start < end && tokens[start].is_whitespace() {
        start += 1;
    }
    while end > start && tokens[end - 1].is_whitespace() {
        end -= 1;
    }
    &tokens[start..end]
}

#[doc = include_str!("property_parser.doc/strip_important.md")]
fn strip_important(tokens: &[CssToken]) -> (&[CssToken], bool) {
    let trimmed = trim_ws(tokens);
    let Some((last, rest)) = trimmed.split_last() else {
        return (tokens, false);
    };
    let is_important = matches!(
        last.kind(),
        CssTokenKind::Ident(id) if id.eq_ignore_ascii_case("important")
    );
    if !is_important {
        return (tokens, false);
    }
    let rest = trim_ws(rest);
    match rest.split_last() {
        Some((bang, before)) if matches!(bang.kind(), CssTokenKind::Delim('!')) => (before, true),
        _ => (tokens, false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Result<ParsedDeclaration, CssParseError> {
        let src = CssSource::anonymous(text);
        PropertyParser::new().parse(&src)
    }

    #[test]
    fn parses_simple_declaration() {
        let decl = parse("color: red").unwrap();
        assert_eq!(&*decl.name, "color");
        assert_eq!(decl.value, vec![ParsedValue::Keyword("red".into())]);
        assert!(!decl.important);
    }

    #[test]
    fn parses_dimension_value() {
        let decl = parse("width: 10px").unwrap();
        assert_eq!(
            decl.value,
            vec![ParsedValue::Dimension {
                value: 10.0,
                unit: "px".into()
            }]
        );
    }

    #[test]
    fn parses_important() {
        let decl = parse("color: red !important").unwrap();
        assert!(decl.important);
        assert_eq!(decl.value, vec![ParsedValue::Keyword("red".into())]);
    }

    #[test]
    fn important_is_case_insensitive() {
        assert!(parse("color: red !IMPORTANT").unwrap().important);
    }

    #[test]
    fn important_with_whitespace() {
        let decl = parse("color: red ! important").unwrap();
        assert!(decl.important);
        assert_eq!(decl.value, vec![ParsedValue::Keyword("red".into())]);
    }

    #[test]
    fn parses_custom_property() {
        let decl = parse("--main-color: #fff").unwrap();
        assert_eq!(&*decl.name, "--main-color");
        assert_eq!(decl.value, vec![ParsedValue::Hash("fff".into())]);
    }

    #[test]
    fn parses_multi_component_value() {
        let decl = parse("margin: 1px 2px 3px 4px").unwrap();
        assert_eq!(decl.value.len(), 4);
    }

    #[test]
    fn parses_function_value() {
        let decl = parse("width: calc(100% - 20px)").unwrap();
        assert!(matches!(decl.value[0], ParsedValue::Function { .. }));
    }

    #[test]
    fn missing_colon_is_error() {
        assert_eq!(
            parse("color red").unwrap_err().kind(),
            CssParseErrorKind::ExpectedColon
        );
    }

    #[test]
    fn missing_value_is_error() {
        assert_eq!(
            parse("color:").unwrap_err().kind(),
            CssParseErrorKind::InvalidValue
        );
    }

    #[test]
    fn missing_name_is_error() {
        assert_eq!(
            parse(": red").unwrap_err().kind(),
            CssParseErrorKind::UnexpectedToken
        );
    }

    #[test]
    fn span_covers_declaration() {
        let decl = parse("color: red").unwrap();
        assert_eq!(decl.span, SourceSpan::new(0, 10));
    }

    #[test]
    fn bang_without_important_is_not_important() {
        // A lone `!` with a different ident is kept in the value, not stripped.
        let decl = parse("content: red !bogus").unwrap();
        assert!(!decl.important);
        assert!(decl.value.contains(&ParsedValue::Delimiter('!')));
    }
}
