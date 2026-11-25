use super::SimpleSelector;

/// A compound selector: contiguous simple selectors with no combinator between
/// them (`div.a#b:hover`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParsedCompoundSelector {
    /// The simple selectors making up the compound, in source order.
    pub parts: Vec<SimpleSelector>,
}
