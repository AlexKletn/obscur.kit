//! The [`ContainerQueryParser`] and its parsed container-query AST (spec §4.13).
//!
//! Parses an `@container` prelude: an optional container name followed by a
//! feature condition list (`(min-width: 400px)`, `(inline-size > 200px)`). The
//! feature/condition grammar is shared with [`super::media_query_parser`].

use crate::error::CssParseError;
use crate::source::CssSource;

use super::media_query_parser::{Inner, MediaComparator, MediaCondition};
use super::stylesheet_parser::{CssToken, Tokenizer};
use super::value_parser::ParsedValue;

/// A parsed `@container` query: an optional name and its feature conditions.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParsedContainerQuery {
    /// The container name, if the query targets a named container.
    pub name: Option<Box<str>>,
    /// The feature conditions, in source order.
    pub conditions: Vec<ContainerCondition>,
}

/// A single container feature condition, mirroring the media condition shape.
#[derive(Debug, Clone, PartialEq)]
pub struct ContainerCondition {
    /// The feature name (for example `min-width`, `inline-size`).
    pub feature: Box<str>,
    /// The range comparator, or [`None`] for a colon or boolean feature.
    pub comparator: Option<MediaComparator>,
    /// The feature value, or [`None`] for a boolean feature.
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

/// Parses an `@container` prelude into a [`ParsedContainerQuery`].
///
/// The parser is stateless; a single instance may parse any number of preludes.
#[derive(Debug, Clone, Default)]
pub struct ContainerQueryParser {
    _private: (),
}

impl ContainerQueryParser {
    /// Creates a container-query parser.
    #[must_use]
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Parses `source` as a container-query prelude (without the `@container`
    /// keyword).
    ///
    /// # Errors
    ///
    /// Returns a [`CssParseError`] with kind
    /// [`InvalidAtRule`](crate::error::CssParseErrorKind::InvalidAtRule) on
    /// malformed syntax.
    pub fn parse(&self, source: &CssSource) -> Result<ParsedContainerQuery, CssParseError> {
        let text = source.text();
        let eof = u32::try_from(text.len()).unwrap_or(u32::MAX);
        let (tokens, mut errors) = Tokenizer::new(text).tokenize();
        if !errors.is_empty() {
            return Err(errors.remove(0));
        }
        Self::parse_tokens(&tokens, eof)
    }

    /// Parses a container-query prelude from an already-tokenized slice.
    ///
    /// # Errors
    ///
    /// Returns a [`CssParseError`] on malformed syntax.
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
