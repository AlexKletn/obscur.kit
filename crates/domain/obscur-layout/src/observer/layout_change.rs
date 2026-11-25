use obscur_identity::NodeId;

use crate::model::LayoutBox;

#[doc = include_str!("../../docs/api/observer/layout_observer.doc/LayoutChange.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutChange {
    /// The observed node.
    pub node: NodeId,
    /// The box before this change, or `None` on the first observation.
    pub previous: Option<LayoutBox>,
    /// The newly-committed box.
    pub current: LayoutBox,
}

impl LayoutChange {
    /// Constructs a layout change event.
    #[must_use]
    pub const fn new(node: NodeId, previous: Option<LayoutBox>, current: LayoutBox) -> Self {
        Self {
            node,
            previous,
            current,
        }
    }
}
