use super::{Combinator, ParsedCompoundSelector};

#[doc = include_str!("../../docs/api/parser/selector_parser.doc/ParsedComplexSelector.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedComplexSelector {
    /// The first compound selector.
    pub head: ParsedCompoundSelector,
    /// The remaining compounds, each with its preceding combinator.
    pub tail: Vec<(Combinator, ParsedCompoundSelector)>,
}
