use super::AttrMatch;

#[doc = include_str!("../../docs/api/selector/simple_selector.doc/AttributeSelector.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeSelector {
    /// The attribute name that must be present.
    pub name: Box<str>,
    /// The value test, or [`None`] for a bare presence test.
    pub matcher: Option<AttrMatch>,
}
