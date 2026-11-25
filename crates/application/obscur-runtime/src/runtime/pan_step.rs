use obscur_geometry::Point;
use obscur_identity::NodeId;

/// One consumed pan move: the container to scroll and by how much.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PanStep {
    /// The container the drag belongs to, resolved when the contact went down.
    pub(crate) container: NodeId,
    /// Scroll displacement in logical pixels on the axis selected when the pan began.
    pub(crate) scroll_delta: Point,
    /// Whether this step discarded an outstanding press with a target — the
    /// caller schedules a restyle for the `:active` that went out.
    pub(crate) press_discarded: bool,
}
