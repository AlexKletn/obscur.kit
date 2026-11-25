use super::{ParsedDeclaration, ParsedSelectorList};

/// A style rule: a selector list paired with its declarations.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedStyleRule {
    /// The rule's selector list.
    pub selectors: ParsedSelectorList,
    /// The declarations in the rule's block.
    pub declarations: Vec<ParsedDeclaration>,
}
