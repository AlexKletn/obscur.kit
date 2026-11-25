use crate::error::{CssParseError, CssParseErrorKind};

use super::{MediaComparator, MediaCondition, MediaLogic, MediaQualifier, ParsedMediaQuery};
use super::{ParsedValue, ValueParser};
use super::{css_token::CssToken, css_token_kind::CssTokenKind};

/// Token cursor and condition parser shared by media and container queries.
pub(super) struct QueryConditionParser<'t> {
    tokens: &'t [CssToken],
    pos: usize,
    eof: u32,
}

impl<'t> QueryConditionParser<'t> {
    pub(crate) fn new(tokens: &'t [CssToken], eof: u32) -> Self {
        Self {
            tokens,
            pos: 0,
            eof,
        }
    }

    pub(super) fn kind(&self) -> Option<&'t CssTokenKind> {
        self.tokens.get(self.pos).map(CssToken::kind)
    }

    pub(super) fn advance(&mut self) {
        self.pos += 1;
    }

    pub(super) fn skip_ws(&mut self) {
        while matches!(self.tokens.get(self.pos), Some(t) if t.is_whitespace()) {
            self.pos += 1;
        }
    }

    pub(super) fn err(&self, message: &str) -> CssParseError {
        let span = self.tokens.get(self.pos).map_or_else(
            || crate::source::SourceSpan::empty(self.eof),
            CssToken::span,
        );
        CssParseError::new(CssParseErrorKind::InvalidAtRule, message, span)
    }

    pub(super) fn parse_query(&mut self) -> Result<ParsedMediaQuery, CssParseError> {
        self.skip_ws();
        let start = self.pos;
        let mut qualifier = self.take_qualifier();
        self.skip_ws();
        let media_type = self.take_media_type();
        if media_type.is_none() {
            if qualifier == Some(MediaQualifier::Only) {
                return Err(self.err("expected a media type after 'only'"));
            }
            self.pos = start;
            qualifier = None;
        }
        self.skip_ws();
        let mut conditions = Vec::new();
        let logic = if media_type.is_some() {
            if self.at_keyword("and") {
                self.advance();
                Some(self.parse_media_logic(0, false, &mut conditions)?)
            } else {
                None
            }
        } else {
            Some(self.parse_media_logic(0, true, &mut conditions)?)
        };
        Ok(ParsedMediaQuery {
            qualifier,
            media_type,
            conditions,
            logic,
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
        if self.at_style_function() {
            return None;
        }
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

    fn at_keyword(&self, word: &str) -> bool {
        matches!(self.kind(), Some(CssTokenKind::Ident(id)) if id.eq_ignore_ascii_case(word))
    }

    /// Parses a unary negation or a homogeneous group of media operators.
    fn parse_media_logic(
        &mut self,
        depth: usize,
        allow_or: bool,
        conditions: &mut Vec<MediaCondition>,
    ) -> Result<MediaLogic, CssParseError> {
        if depth >= 32 {
            return Err(self.err("media condition nesting limit exceeded"));
        }
        self.skip_ws();
        if self.at_keyword("not") {
            self.advance();
            return self
                .parse_media_operand(depth, conditions)
                .map(|operand| MediaLogic::Not(Box::new(operand)));
        }
        let first = self.parse_media_operand(depth, conditions)?;
        self.skip_ws();
        let or = self.at_keyword("or");
        if !or && !self.at_keyword("and") {
            return Ok(first);
        }
        if or && !allow_or {
            return Err(self.err("'or' after a media type must be parenthesized"));
        }
        let connector = if or { "or" } else { "and" };
        let mut parts = vec![first];
        while self.at_keyword(connector) {
            self.advance();
            parts.push(self.parse_media_operand(depth, conditions)?);
            self.skip_ws();
        }
        if self.at_keyword("and") || self.at_keyword("or") {
            return Err(self.err("mixed media operators require parentheses"));
        }
        Ok(if or {
            MediaLogic::Any(parts)
        } else {
            MediaLogic::All(parts)
        })
    }

    /// Parses one nested media expression or one media feature, including range bounds.
    fn parse_media_operand(
        &mut self,
        depth: usize,
        conditions: &mut Vec<MediaCondition>,
    ) -> Result<MediaLogic, CssParseError> {
        self.skip_ws();
        if !matches!(self.kind(), Some(CssTokenKind::LParen)) {
            return Err(self.err("expected a parenthesized media condition"));
        }
        let start = self.pos;
        self.advance();
        self.skip_ws();
        if matches!(self.kind(), Some(CssTokenKind::LParen)) || self.at_keyword("not") {
            let logic = self.parse_media_logic(depth + 1, true, conditions)?;
            self.skip_ws();
            if !matches!(self.kind(), Some(CssTokenKind::RParen)) {
                return Err(self.err("expected ')' after the media condition group"));
            }
            self.advance();
            return Ok(logic);
        }
        self.pos = start;
        let first = conditions.len();
        self.parse_feature(conditions)?;
        if conditions.len() == first + 1 {
            Ok(MediaLogic::Leaf(first))
        } else {
            Ok(MediaLogic::All(
                (first..conditions.len()).map(MediaLogic::Leaf).collect(),
            ))
        }
    }

    pub(super) fn at_style_function(&self) -> bool {
        self.at_function("style")
    }

    pub(super) fn at_function(&self, name: &str) -> bool {
        matches!(self.kind(), Some(CssTokenKind::Ident(actual)) if actual.eq_ignore_ascii_case(name))
            && matches!(
                self.tokens.get(self.pos + 1).map(CssToken::kind),
                Some(CssTokenKind::LParen)
            )
    }

    fn parse_feature(&mut self, conditions: &mut Vec<MediaCondition>) -> Result<(), CssParseError> {
        self.pos += 1;
        self.skip_ws();
        if matches!(
            self.kind(),
            Some(CssTokenKind::Number(_) | CssTokenKind::Dimension { .. })
        ) || matches!(self.kind(), Some(CssTokenKind::Ident(name)) if name.eq_ignore_ascii_case("infinite"))
        {
            return self.parse_value_first_feature(conditions);
        }
        conditions.push(self.parse_feature_body()?);
        Ok(())
    }

    pub(super) fn parse_feature_body(&mut self) -> Result<MediaCondition, CssParseError> {
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

    /// Parses value-first media ranges and lowers each bound to a comparison against the feature. A chained range requires matching comparison directions.
    fn parse_value_first_feature(
        &mut self,
        conditions: &mut Vec<MediaCondition>,
    ) -> Result<(), CssParseError> {
        let value = ValueParser::parse_tokens(&self.tokens[self.pos..self.pos + 1], self.eof)?;
        self.pos += 1;
        self.skip_ws();
        let left = self
            .take_comparator()
            .ok_or_else(|| self.err("expected a comparator after the feature value"))?;
        let comparator = match left {
            MediaComparator::Lt => MediaComparator::Gt,
            MediaComparator::Le => MediaComparator::Ge,
            MediaComparator::Eq => MediaComparator::Eq,
            MediaComparator::Gt => MediaComparator::Lt,
            MediaComparator::Ge => MediaComparator::Le,
        };
        self.skip_ws();
        let Some(CssTokenKind::Ident(feature)) = self.kind() else {
            return Err(self.err("expected a feature name after the comparator"));
        };
        let feature = feature.clone();
        self.pos += 1;
        self.skip_ws();
        conditions.push(MediaCondition {
            feature: feature.clone(),
            comparator: Some(comparator),
            value: Some(value),
        });

        if matches!(self.kind(), Some(CssTokenKind::RParen)) {
            self.pos += 1;
            return Ok(());
        }
        let right = self.take_comparator().ok_or_else(|| {
            self.err("expected ')' or a second comparator after the feature name")
        })?;
        use MediaComparator::{Ge, Gt, Le, Lt};
        if !matches!((left, right), (Lt | Le, Lt | Le) | (Gt | Ge, Gt | Ge)) {
            return Err(self.err("range comparisons must point in the same direction"));
        }
        let value = self.parse_value_until_rparen()?;
        conditions.push(MediaCondition {
            feature,
            comparator: Some(right),
            value: Some(value),
        });
        Ok(())
    }

    pub(super) fn take_comparator(&mut self) -> Option<MediaComparator> {
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
    pub(super) fn parse_value_until_rparen(&mut self) -> Result<Vec<ParsedValue>, CssParseError> {
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
pub(super) fn is_connector(id: &str) -> bool {
    id.eq_ignore_ascii_case("and")
        || id.eq_ignore_ascii_case("or")
        || id.eq_ignore_ascii_case("not")
}
