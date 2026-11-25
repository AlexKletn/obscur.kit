use super::{ParsedContainerQuery, ParsedRule};

/// An `@container` rule: a container query wrapping a nested rule list.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedContainerRule {
    /// The container query from the rule's prelude.
    pub query: ParsedContainerQuery,
    /// The nested rules the container block applies to.
    pub rules: Vec<ParsedRule>,
}
