/// A combinator linking two compound selectors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combinator {
    /// Descendant combinator (whitespace).
    Descendant,
    /// Child combinator (`>`).
    Child,
    /// Next-sibling combinator (`+`).
    NextSibling,
    /// Subsequent-sibling combinator (`~`).
    SubsequentSibling,
}
