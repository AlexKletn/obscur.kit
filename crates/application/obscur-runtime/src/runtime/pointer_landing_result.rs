use obscur_geometry::Point;
use obscur_identity::NodeId;

#[doc = include_str!("../../docs/api/runtime/pointer_landing.doc/PointerLanding.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PointerLanding {
    #[doc = include_str!("../../docs/api/runtime/pointer_landing.doc/screen.md")]
    pub(crate) screen: Point,
    #[doc = include_str!("../../docs/api/runtime/pointer_landing.doc/folded.md")]
    pub(crate) folded: Point,
    /// The node this event addresses: its captor if the pointer is captured, else
    /// the topmost hit node, else [`None`] for a landing on empty space.
    pub(crate) target: Option<NodeId>,
}
