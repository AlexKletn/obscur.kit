#[doc = include_str!("../../docs/api/selector/complex_selector.doc/Combinator.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combinator {
    /// Descendant combinator (whitespace): the right compound is a descendant
    /// of the left.
    Descendant,
    /// Child combinator (`>`): the right compound is a direct child of the left.
    Child,
    /// Next-sibling combinator (`+`): the right compound immediately follows the
    /// left among element siblings.
    NextSibling,
    /// Subsequent-sibling combinator (`~`): the right compound follows the left
    /// among element siblings (not necessarily immediately).
    SubsequentSibling,
}
