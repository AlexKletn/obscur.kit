use crate::error::{CssParseError, UnsupportedCssFeature};

use super::ParsedRule;

#[doc = include_str!("../../docs/api/parser/stylesheet_parser.doc/ParsedStylesheet.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedStylesheet {
    /// The recovered top-level rules, in source order.
    pub rules: Vec<ParsedRule>,
    /// Syntax errors encountered (each rule after an error is still attempted).
    pub errors: Vec<CssParseError>,
    /// Recognised-but-unsupported at-rules encountered during parsing.
    pub unsupported: Vec<UnsupportedCssFeature>,
}
