#[doc = include_str!("../../docs/api/orchestration/stacking.doc/PaintLayer.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum PaintLayer {
    /// §9.9 layers 1/3/4/5 — the context root's bg+borders and all non-positioned
    /// in-flow descendants (collapsed; see the type docs). Sorts *below*
    /// [`PositionedZeroOrAuto`](Self::PositionedZeroOrAuto).
    NonPositioned,
    /// §9.9 layer 6 — a positioned descendant with effective own `z-index` `auto`
    /// or `0` (and its subtree). Sorts *above* [`NonPositioned`](Self::NonPositioned)
    /// within the same z bucket.
    PositionedZeroOrAuto,
}
