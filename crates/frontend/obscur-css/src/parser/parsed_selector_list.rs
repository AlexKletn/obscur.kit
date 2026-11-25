use super::ParsedComplexSelector;

/// A comma-separated list of complex selectors (`a, b > c`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParsedSelectorList(pub Vec<ParsedComplexSelector>);
