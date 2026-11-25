use super::{max_track_fn::MaxFn, min_track_fn::MinFn};

/// One track's two-value sizing state during the §11 pipeline.
pub(super) struct Track {
    /// The growing floor (§11 "base size").
    pub(super) base: f32,
    /// The ceiling (§11 "growth limit"), `f32::INFINITY` while unbounded.
    pub(super) limit: f32,
    pub(super) min_fn: MinFn,
    pub(super) max_fn: MaxFn,
    pub(super) fit_limit: f32,
    /// Scratch: this item's increase within one distribution round.
    pub(super) increase: f32,
    /// Scratch: the largest per-item base increase planned across a sub-step.
    pub(super) planned_base: f32,
    /// Scratch: the largest per-item growth-limit increase planned across a
    /// sub-step.
    pub(super) planned_limit: f32,
    /// Whether this track's growth limit turned finite in the current
    /// growth-limit round (§11.5.1 "infinitely growable" flag).
    pub(super) infinitely_growable: bool,
    #[doc = include_str!("../../docs/api/algorithm/grid_track_sizing.doc/max_touched.md")]
    pub(super) max_touched: bool,
    /// Whether this track's *max* sizing function is literally `auto` (as opposed
    /// to `max-content`, which shares the same [`Kind::Max`](super::intrinsic_track_kind::Kind::Max) content contribution).
    /// §11.8 *Stretch auto Tracks* grows only `auto`-max tracks, so this preserves
    /// the distinction the growth-limit [`Kind`](super::intrinsic_track_kind::Kind) no longer carries.
    pub(super) auto_max: bool,
}
