use obscur_geometry::Point;
use obscur_identity::NodeId;

/// The last `Down` press, retained so a rapid run of presses on the same target
/// near the same point is recognised as a double/triple *click* (spec §5.11
/// editing engine — word/line selection via the pointer).
#[derive(Debug, Clone, Copy)]
pub(super) struct PointerClick {
    /// The interactive target the press landed on (`None` for empty space).
    pub(super) target: Option<NodeId>,
    /// The press point in content space.
    pub(super) point: Point,
    /// The press's logical timestamp, in microseconds.
    pub(super) time_micros: u64,
    /// The running click count of this press (`1`, `2`, or `3`, then cycling).
    pub(super) count: u32,
}
