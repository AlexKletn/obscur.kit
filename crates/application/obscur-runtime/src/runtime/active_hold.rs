use obscur_geometry::Point;
use obscur_identity::{InternedSymbol, NodeId};
use obscur_interaction::gesture::LongPressRecognizer;

#[doc = include_str!("../../docs/api/runtime/pointer_sessions.doc/ActiveHold.md")]
#[derive(Debug, Clone, Copy)]
pub(crate) struct ActiveHold {
    pub(crate) press: LongPressRecognizer,
    pub(crate) target: NodeId,
    pub(crate) origin: Point,
    pub(crate) trusted: bool,
    /// The virtual list and model index the contact went down on, if it landed on
    /// a realized row. Resolved once, at contact time: the row under the finger
    /// is the row being held, and the list cannot scroll while that finger is the
    /// one holding it still.
    pub(crate) row: Option<(InternedSymbol, usize)>,
}
