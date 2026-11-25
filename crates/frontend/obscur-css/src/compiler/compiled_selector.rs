use obscur_style::selector::{SelectorId, SelectorSpecificity};

#[doc = include_str!("../../docs/api/compiler/selector_compiler.doc/CompiledSelector.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CompiledSelector {
    /// The sequential handle minted for this selector.
    pub id: SelectorId,
    /// The computed CSS `(a, b, c)` specificity.
    pub specificity: SelectorSpecificity,
}
