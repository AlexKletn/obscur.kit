use obscur_geometry::Size;

use crate::model::{LayoutBox, LayoutStyle, Margin};

use super::{ChildMeasurer, LayoutChild};

/// Inputs for [`place_line`](super::flex_line_placement::place_line): one flex line's items plus the shared container /
/// style context. Grouped into a struct because the single-line sizing pass
/// legitimately needs this many parameters and a positional argument list would
/// be error-prone.
pub(super) struct LineArgs<'a, 'm> {
    /// The item indices on this line, in source order.
    pub(super) line: &'a [usize],
    /// Hypothetical main size per item (flex base clamped to content min/max),
    /// indexed by the global item index. Drives line-breaking and the
    /// freeze-inflexible test in the §9.7 resolve-flexible-lengths loop.
    pub(super) base_main: &'a [f32],
    /// Unclamped flex base size per item, indexed by the global item index. This
    /// is what grow/shrink distribute from in the §9.7 freeze loop.
    pub(super) flex_base: &'a [f32],
    /// Pre-resolved main-axis min floor / max ceiling per item, indexed by the
    /// global item index (`main_max` is `f32::INFINITY` when the item has no
    /// `max`). `main_min` folds in the §4.5 content-based automatic minimum on the
    /// `auto` path, so the freeze loop shrinks items no further than their content.
    pub(super) main_min: &'a [f32],
    pub(super) main_max: &'a [f32],
    /// Whether each item's §4.5 content floor was deferred (a `content.min(fb)`
    /// floor whose content measure is taken here, lazily, only when this line
    /// shrinks). Indexed by the global item index.
    pub(super) deferred_floor: &'a [bool],
    /// The flex items.
    pub(super) children: &'a [LayoutChild],
    /// The flex container's style.
    pub(super) style: &'a LayoutStyle,
    /// Whether the main axis is horizontal.
    pub(super) is_row: bool,
    /// Whether the logical main-axis cursor is measured from the right or bottom edge.
    pub(super) is_reverse: bool,
    /// The main-axis extent this line's free space / justification resolve against.
    pub(super) line_main: f32,
    /// Gap between adjacent items on the line.
    pub(super) gap: f32,
    /// Leading / trailing main-axis margins, indexed by the global item index.
    pub(super) main_lead: &'a [Margin],
    pub(super) main_trail: &'a [Margin],
    /// Leading cross-axis margin, indexed by the global item index.
    pub(super) cross_lead: &'a [Margin],
    /// When `Some(target)`, `stretch` fills `target` and alignment resolves
    /// against `container_cross` (the single-line / no-wrap semantics). When
    /// `None`, the line's cross size is the max item cross size and `stretch`
    /// fills that (the multi-line semantics).
    pub(super) stretch_target: Option<f32>,
    /// The container's cross extent (the measurement bound and, for the
    /// single-line path, the alignment reference).
    pub(super) container_cross: f32,
    #[doc = include_str!("../../docs/api/algorithm/flex_line_placement.doc/percent_basis.md")]
    pub(super) percent_basis: Size,
    /// Content-box origin offsets.
    pub(super) bp_left: f32,
    pub(super) bp_top: f32,
    /// The line's leading cross-axis offset (0 for a single line).
    pub(super) cross_offset: f32,
    /// Whether the caller needs [`PlacedLine::content_cross`](super::placed_line::PlacedLine::content_cross) (the line's
    /// pre-stretch content cross extent). When `false`, a stretched item's
    /// hypothetical cross is not measured, keeping the definite path byte-identical
    /// and avoiding an unnecessary measurement.
    pub(super) content_cross_needed: bool,
    /// Multi-line only (`stretch_target == None`): when `Some(target)`, the line's
    /// cross box is grown to `max(natural, target)` and `align-items: stretch`
    /// items fill the grown cross. This is how `align-content: stretch` grows each
    /// flex line by its share of the container's leftover cross space
    /// (css-flexbox-1 §8.4). `None` keeps the natural (max item cross) sizing.
    pub(super) line_cross_stretch: Option<f32>,
    /// Child measurer.
    pub(super) measurer: &'m mut dyn ChildMeasurer,
    /// Output boxes, written at each item's global index.
    pub(super) child_boxes: &'a mut [LayoutBox],
}
