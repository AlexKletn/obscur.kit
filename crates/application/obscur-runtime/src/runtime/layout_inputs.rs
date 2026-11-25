//! What layout reads that is not a computed style: [`LayoutInputs`].

use obscur_geometry::Size;
use obscur_identity::NodeId;

#[doc = include_str!("../../docs/api/runtime/layout_inputs.doc/LayoutInputs.md")]
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LayoutInputs {
    /// The viewport the boxes were arranged under.
    pub(crate) viewport: Size,
    /// The viewport scroll offset (drives `position: sticky`).
    pub(crate) scroll: (f32, f32),
    /// The focused editable's `(node, text, preedit)` — the string the text leaf
    /// was measured from; `None` when no editing session is active.
    pub(crate) editing: Option<(NodeId, String, Option<String>)>,
    #[doc = include_str!("../../docs/api/runtime/layout_inputs.doc/language.md")]
    pub(crate) language: Box<str>,
}
