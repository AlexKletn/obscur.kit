#![doc = include_str!("container_query_parser.doc/_module.md")]

use crate::error::CssParseError;
use crate::source::CssSource;

use super::css_tokenizer::{CssToken, Tokenizer};
use super::media_query_parser::{Inner, MediaComparator, MediaCondition};
use super::value_parser::ParsedValue;

#[doc = include_str!("container_query_parser.doc/ParsedContainerQuery.md")]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParsedContainerQuery {
    #[doc = include_str!("container_query_parser.doc/name.md")]
    pub name: Option<Box<str>>,
    #[doc = include_str!("container_query_parser.doc/conditions.md")]
    pub conditions: Vec<ContainerCondition>,
}

#[doc = include_str!("container_query_parser.doc/ContainerCondition.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct ContainerCondition {
    #[doc = include_str!("container_query_parser.doc/feature.md")]
    pub feature: Box<str>,
    #[doc = include_str!("container_query_parser.doc/comparator.md")]
    pub comparator: Option<MediaComparator>,
    #[doc = include_str!("container_query_parser.doc/value.md")]
    pub value: Option<Vec<ParsedValue>>,
}

impl From<MediaCondition> for ContainerCondition {
    fn from(condition: MediaCondition) -> Self {
        Self {
            feature: condition.feature,
            comparator: condition.comparator,
            value: condition.value,
        }
    }
}

#[doc = include_str!("container_query_parser.doc/ContainerQueryParser.md")]
#[derive(Debug, Clone, Default)]
pub struct ContainerQueryParser {
    _private: (),
}

impl ContainerQueryParser {
    #[doc = include_str!("container_query_parser.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self { _private: () }
    }

    #[doc = include_str!("container_query_parser.doc/parse.md")]
    pub fn parse(&self, source: &CssSource) -> Result<ParsedContainerQuery, CssParseError> {
        let text = source.text();
        let eof = u32::try_from(text.len()).unwrap_or(u32::MAX);
        let (tokens, mut errors) = Tokenizer::new(text).tokenize();
        if !errors.is_empty() {
            return Err(errors.remove(0));
        }
        Self::parse_tokens(&tokens, eof)
    }

    #[doc = include_str!("container_query_parser.doc/parse_tokens.md")]
    pub(crate) fn parse_tokens(
        tokens: &[CssToken],
        eof: u32,
    ) -> Result<ParsedContainerQuery, CssParseError> {
        let mut inner = Inner::new(tokens, eof);
        let name = inner.take_container_name();
        let conditions = inner
            .parse_conditions()?
            .into_iter()
            .map(ContainerCondition::from)
            .collect();
        Ok(ParsedContainerQuery { name, conditions })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::CssParseErrorKind;

    fn parse(text: &str) -> Result<ParsedContainerQuery, CssParseError> {
        let src = CssSource::anonymous(text);
        ContainerQueryParser::new().parse(&src)
    }

    #[test]
    fn parses_unnamed_condition() {
        let q = parse("(min-width: 400px)").unwrap();
        assert!(q.name.is_none());
        assert_eq!(q.conditions.len(), 1);
        assert_eq!(&*q.conditions[0].feature, "min-width");
        assert_eq!(
            q.conditions[0].value,
            Some(vec![ParsedValue::Dimension {
                value: 400.0,
                unit: "px".into()
            }])
        );
    }

    #[test]
    fn parses_named_container() {
        let q = parse("sidebar (min-width: 400px)").unwrap();
        assert_eq!(q.name.as_deref(), Some("sidebar"));
        assert_eq!(q.conditions.len(), 1);
    }

    #[test]
    fn parses_range_condition() {
        let q = parse("(inline-size > 200px)").unwrap();
        assert_eq!(&*q.conditions[0].feature, "inline-size");
        assert_eq!(q.conditions[0].comparator, Some(MediaComparator::Gt));
    }

    #[test]
    fn parses_named_range_condition() {
        let q = parse("layout (inline-size >= 200px)").unwrap();
        assert_eq!(q.name.as_deref(), Some("layout"));
        assert_eq!(q.conditions[0].comparator, Some(MediaComparator::Ge));
    }

    #[test]
    fn parses_multiple_conditions() {
        let q = parse("(min-width: 400px) and (max-width: 800px)").unwrap();
        assert_eq!(q.conditions.len(), 2);
    }

    #[test]
    fn parses_boolean_condition() {
        let q = parse("(orientation: landscape)").unwrap();
        assert_eq!(
            q.conditions[0].value,
            Some(vec![ParsedValue::Keyword("landscape".into())])
        );
    }

    #[test]
    fn not_keyword_is_not_a_name() {
        // `not` is a connector, so it must not be captured as a container name.
        let q = parse("not (min-width: 400px)").unwrap();
        assert!(q.name.is_none());
    }

    #[test]
    fn unterminated_condition_errors() {
        assert_eq!(
            parse("(min-width: 400px").unwrap_err().kind(),
            CssParseErrorKind::InvalidAtRule
        );
    }
}
