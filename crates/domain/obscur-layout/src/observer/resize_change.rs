use obscur_geometry::Size;
use obscur_identity::NodeId;

#[doc = include_str!("../../docs/api/observer/resize_observer.doc/ResizeChange.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResizeChange {
    /// The observed node.
    pub node: NodeId,
    /// The size before this change, or `None` on the first observation.
    pub previous: Option<Size>,
    /// The newly-committed size.
    pub current: Size,
}

impl ResizeChange {
    /// Constructs a resize change event.
    #[must_use]
    pub const fn new(node: NodeId, previous: Option<Size>, current: Size) -> Self {
        Self {
            node,
            previous,
            current,
        }
    }
}
