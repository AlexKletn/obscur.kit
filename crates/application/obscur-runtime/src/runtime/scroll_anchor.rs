//! Explicit scroll anchoring for retained layout.

use obscur_geometry::Point;
use obscur_identity::NodeId;

/// A retained descendant and its position relative to a scroll viewport.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollAnchor {
    pub(super) container: NodeId,
    pub(super) node: NodeId,
    pub(super) viewport_position: Point,
}
