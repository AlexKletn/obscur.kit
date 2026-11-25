#[doc = include_str!("../../docs/api/algorithm/grid_track_sizing.doc/Kind.md")]
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    /// An `auto` track: the item's size under the container's own loose offer
    /// (the documented approximation — see the module note above).
    Auto,
    /// A `min-content` track: the item measured under an offer of no space.
    Min,
    /// A `max-content` track: the item measured under an unbounded offer.
    Max,
}

/// Every intrinsic kind, in the order §11.5 works through them: the smallest
/// contribution first, so a track sized by a larger one is never grown twice.
pub(super) const KINDS: [Kind; 3] = [Kind::Auto, Kind::Min, Kind::Max];
