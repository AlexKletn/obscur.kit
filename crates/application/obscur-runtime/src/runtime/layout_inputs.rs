#![doc = include_str!("layout_inputs.doc/_module.md")]

use obscur_geometry::Size;
use obscur_identity::NodeId;

#[doc = include_str!("layout_inputs.doc/LayoutInputs.md")]
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LayoutInputs {
    #[doc = include_str!("layout_inputs.doc/viewport.md")]
    pub(crate) viewport: Size,
    #[doc = include_str!("layout_inputs.doc/scroll.md")]
    pub(crate) scroll: (f32, f32),
    #[doc = include_str!("layout_inputs.doc/editing.md")]
    pub(crate) editing: Option<(NodeId, String, Option<String>)>,
    #[doc = include_str!("layout_inputs.doc/language.md")]
    pub(crate) language: Box<str>,
}
