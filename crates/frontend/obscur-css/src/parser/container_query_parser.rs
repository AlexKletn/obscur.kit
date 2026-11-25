#![doc = include_str!("../../docs/api/parser/container_query_parser.doc/_module.md")]

use crate::error::CssParseError;
use crate::source::CssSource;

use super::query_condition_parser::{QueryConditionParser, is_connector};
use super::{
    ContainerCondition, ContainerGroup, ContainerLogic, MediaComparator, ParsedContainerQuery,
    ParsedValue,
};
use super::{css_token::CssToken, css_token_kind::CssTokenKind, css_tokenizer::Tokenizer};

#[doc = include_str!("../../docs/api/parser/container_query_parser.doc/ContainerQueryParser.md")]
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

    #[doc = include_str!("../../docs/api/parser/container_query_parser.doc/parse.md")]
    pub fn parse(&self, source: &CssSource) -> Result<ParsedContainerQuery, CssParseError> {
        let text = source.text();
        let eof = u32::try_from(text.len()).unwrap_or(u32::MAX);
        let (tokens, mut errors) = Tokenizer::new(text).tokenize();
        if !errors.is_empty() {
            return Err(errors.remove(0));
        }
        Self::parse_tokens(&tokens, eof)
    }

    #[doc = include_str!("../../docs/api/parser/container_query_parser.doc/parse_tokens.md")]
    pub(crate) fn parse_tokens(
        tokens: &[CssToken],
        eof: u32,
    ) -> Result<ParsedContainerQuery, CssParseError> {
        let mut inner = QueryConditionParser::new(tokens, eof);
        let name = inner.take_container_name();
        let (logic, conditions) = inner.parse_container_conditions()?;
        Ok(ParsedContainerQuery {
            name,
            conditions,
            logic,
        })
    }
}

impl QueryConditionParser<'_> {
    fn parse_container_conditions(
        &mut self,
    ) -> Result<(ContainerLogic, Vec<ContainerCondition>), CssParseError> {
        self.skip_ws();
        let mut conditions = Vec::new();
        if self.kind().is_none() {
            return Ok((ContainerLogic::All, conditions));
        }
        let group = self.parse_container_group(false, 0, &mut conditions)?;
        if self.kind().is_some() {
            return Err(self.err("expected the end of the container query"));
        }
        Ok((group.into_logic(), conditions))
    }

    fn parse_container_group(
        &mut self,
        style: bool,
        depth: usize,
        conditions: &mut Vec<ContainerCondition>,
    ) -> Result<ContainerGroup, CssParseError> {
        if depth == 64 {
            return Err(self.err("container query nesting is too deep"));
        }
        self.skip_ws();
        if matches!(self.kind(), Some(CssTokenKind::Ident(word)) if word.eq_ignore_ascii_case("not"))
        {
            self.advance();
            let operand = self.parse_container_operand(style, depth, conditions)?;
            self.skip_ws();
            return Ok(ContainerGroup::Not(Box::new(operand)));
        }
        let first = self.parse_container_operand(style, depth, conditions)?;
        let mut children = Vec::new();
        let mut disjunction = None;
        loop {
            self.skip_ws();
            let or = match self.kind() {
                None | Some(CssTokenKind::RParen) => break,
                Some(CssTokenKind::Ident(word)) if word.eq_ignore_ascii_case("and") => false,
                Some(CssTokenKind::Ident(word)) if word.eq_ignore_ascii_case("or") => true,
                _ => return Err(self.err("expected 'and' or 'or' between container conditions")),
            };
            if disjunction.is_some_and(|previous| previous != or) {
                return Err(self.err("group conditions before mixing 'and' and 'or'"));
            }
            disjunction = Some(or);
            self.advance();
            children.push(self.parse_container_operand(style, depth, conditions)?);
        }
        let Some(or) = disjunction else {
            return Ok(first);
        };
        children.insert(0, first);
        Ok(if or {
            ContainerGroup::Any(children)
        } else {
            ContainerGroup::All(children)
        })
    }

    fn parse_container_operand(
        &mut self,
        mut style: bool,
        depth: usize,
        conditions: &mut Vec<ContainerCondition>,
    ) -> Result<ContainerGroup, CssParseError> {
        self.skip_ws();
        if !style && self.at_style_function() {
            style = true;
            self.advance();
        }
        if !matches!(self.kind(), Some(CssTokenKind::LParen)) {
            return Err(self.err("expected a parenthesized condition or style()"));
        }
        self.advance();
        self.skip_ws();
        if style {
            if let Some(bound) = numeric_bound(self.kind()) {
                self.advance();
                return self.parse_style_range(bound, conditions);
            }
            if let Some(CssTokenKind::Ident(name)) = self.kind()
                && is_range_function(name)
                && self.at_function(name)
            {
                let name = name.clone();
                self.advance();
                self.advance();
                let args = self.parse_value_until_rparen()?;
                return self.parse_style_range(ParsedValue::Function { name, args }, conditions);
            }
        }
        if matches!(self.kind(), Some(CssTokenKind::Ident(word)) if !is_connector(word))
            && !self.at_style_function()
        {
            let mut condition = ContainerCondition::from(self.parse_feature_body()?);
            condition.style = style;
            if style
                && (condition.feature.as_ref() == "--"
                    || (condition.comparator.is_some() && !condition.feature.starts_with("--")))
            {
                return Err(self.err("expected a property name or declaration inside style()"));
            }
            let index = conditions.len();
            conditions.push(condition);
            return Ok(ContainerGroup::Leaf(index));
        }
        let group = self.parse_container_group(style, depth + 1, conditions)?;
        if !matches!(self.kind(), Some(CssTokenKind::RParen)) {
            return Err(self.err("expected ')' after the container group"));
        }
        self.advance();
        Ok(group)
    }

    fn parse_style_range(
        &mut self,
        bound: ParsedValue,
        conditions: &mut Vec<ContainerCondition>,
    ) -> Result<ContainerGroup, CssParseError> {
        self.skip_ws();
        let Some(left) = self.take_comparator() else {
            return Err(self.err("expected a comparison after the first style range bound"));
        };
        self.skip_ws();
        let Some(CssTokenKind::Ident(name)) = self.kind() else {
            return Err(self.err("expected a custom property in the style range"));
        };
        if !name.starts_with("--") || name.len() == 2 {
            return Err(self.err("expected a custom property in the style range"));
        }
        let first = ContainerCondition {
            style: true,
            feature: name.clone(),
            comparator: Some(reverse_comparator(left)),
            value: Some(vec![bound]),
        };
        self.advance();
        self.skip_ws();
        let index = conditions.len();
        if matches!(self.kind(), Some(CssTokenKind::RParen)) {
            self.advance();
            conditions.push(first);
            return Ok(ContainerGroup::Leaf(index));
        }
        let Some(right) = self.take_comparator() else {
            return Err(self.err("expected a comparison or ')' after the style range property"));
        };
        if !same_range_direction(left, right) {
            return Err(self.err("chained comparisons must point in the same direction"));
        }
        let right_value = self.parse_value_until_rparen()?;
        if !match right_value.as_slice() {
            [
                ParsedValue::Number(_) | ParsedValue::Dimension { .. } | ParsedValue::Percentage(_),
            ] => true,
            [ParsedValue::Function { name, .. }] => is_range_function(name),
            _ => false,
        } {
            return Err(self.err("expected a numeric second style range bound"));
        }
        let second = ContainerCondition {
            style: true,
            feature: first.feature.clone(),
            comparator: Some(right),
            value: Some(right_value),
        };
        conditions.extend([first, second]);
        Ok(ContainerGroup::Range([index, index + 1]))
    }
}

fn is_range_function(name: &str) -> bool {
    ["var", "calc", "min", "max", "clamp"]
        .iter()
        .any(|function| name.eq_ignore_ascii_case(function))
}

fn numeric_bound(token: Option<&CssTokenKind>) -> Option<ParsedValue> {
    match token? {
        CssTokenKind::Number(value) => Some(ParsedValue::Number(*value)),
        CssTokenKind::Percentage(value) => Some(ParsedValue::Percentage(*value)),
        CssTokenKind::Dimension { value, unit } => Some(ParsedValue::Dimension {
            value: *value,
            unit: unit.clone(),
        }),
        _ => None,
    }
}

fn reverse_comparator(comparator: MediaComparator) -> MediaComparator {
    use MediaComparator::*;
    match comparator {
        Lt => Gt,
        Le => Ge,
        Eq => Eq,
        Ge => Le,
        Gt => Lt,
    }
}

fn same_range_direction(left: MediaComparator, right: MediaComparator) -> bool {
    use MediaComparator::*;
    matches!((left, right), (Lt | Le, Lt | Le) | (Gt | Ge, Gt | Ge))
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
    fn parses_custom_property_styles_and_rejects_unsupported_combinations() {
        let query = parse("layout style(--mode: compact) and (width > 10px)").unwrap();
        assert_eq!(query.name.as_deref(), Some("layout"));
        assert!(query.conditions[0].style);
        assert_eq!(query.conditions[0].feature.as_ref(), "--mode");
        assert!(!query.conditions[1].style);
        assert!(parse("style(--present)").unwrap().name.is_none());
        assert_eq!(
            parse("style(--x) or style(--y)").unwrap().logic,
            ContainerLogic::Any
        );
        assert_eq!(parse("not style(--x)").unwrap().logic, ContainerLogic::Not);
        for input in [
            "style(--: red)",
            "style(color > 1px)",
            "style(--x) and",
            "style(--x) or style(--y) and style(--z)",
            "not style(--x) or style(--y)",
        ] {
            assert!(parse(input).is_err(), "{input}");
        }
    }

    #[test]
    fn parses_reversed_and_chained_style_ranges() {
        let reversed = parse("style(12px <= --gap)").unwrap();
        assert_eq!(reversed.conditions[0].comparator, Some(MediaComparator::Ge));
        let chained = parse("style(40px > --gap >= 12px)").unwrap();
        assert_eq!(
            chained.logic,
            ContainerLogic::Nested(Box::new(ContainerGroup::Range([0, 1])))
        );
        assert_eq!(chained.conditions[0].comparator, Some(MediaComparator::Lt));
        assert_eq!(chained.conditions[1].comparator, Some(MediaComparator::Ge));
        for input in [
            "style(12px <= --gap >= 40px)",
            "style(12px = --gap < 40px)",
            "style(12px < --gap <)",
            "style(12px < width)",
            "style(12px < --gap < 40px < 50px)",
        ] {
            assert!(parse(input).is_err(), "{input}");
        }
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
