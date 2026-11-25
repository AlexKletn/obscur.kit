use obscur_scene::command::PaintCommand;

#[doc = include_str!("../../docs/api/orchestration/stacking.doc/PaintUnit.md")]
pub(super) struct PaintUnit {
    /// Each node's effective stacking `z`.
    pub z: i32,
    /// The node's pre-order document position (for a `PopClip` bracket, the
    /// subtree's *exit* index), tie-breaking equal-z units so ties keep tree order.
    pub tree_index: usize,
    /// The within-position phase (see [`PaintUnit::PAINT_ORDER`]).
    pub order: i64,
    /// The node's own paint commands (or the clip bracket), in emission order.
    pub commands: Vec<PaintCommand>,
}

impl PaintUnit {
    /// Opens a CSS filter layer inside the CSS clip, so clipping applies to the filtered result.
    pub(super) const BLUR_PUSH_ORDER: i64 = -3;
    /// Opens the transform around the subtree and its compositing effects.
    pub(super) const TRANSFORM_PUSH_ORDER: i64 = -6;
    /// Opens the opacity or blend group inside the transform and around the CSS clip.
    pub(super) const LAYER_PUSH_ORDER: i64 = -5;
    /// A subtree's opening `PushClip`, sorted before the node's own paint at the
    /// same document position. Pre-order enter indices are unique per node, so
    /// pushes never collide; any value below [`PAINT_ORDER`](Self::PAINT_ORDER)
    /// works.
    pub(super) const CLIP_PUSH_ORDER: i64 = -1;
    /// A node's own paint (background / border / text). The default phase.
    pub(super) const PAINT_ORDER: i64 = 0;

    /// Opens the padding clip after the container paints and before scrolling descendants.
    pub(super) const OVERFLOW_CLIP_PUSH_ORDER: i64 = 1;

    #[doc = include_str!("../../docs/api/orchestration/stacking.doc/SCROLL_TRANSFORM_PUSH_ORDER.md")]
    pub(super) const SCROLL_TRANSFORM_PUSH_ORDER: i64 = 2;

    pub(super) const CSS_CLIP_PUSH_ORDER: i64 = -4;

    // Exit phases unwind the push order. A descendant completes every phase
    // before its parent when both subtrees end at the same paint index.
    pub(super) const fn exit_order(depth: u32, phase: i64) -> i64 {
        i64::MAX - (depth as i64 + 1) * 16 + phase
    }

    pub(super) const fn pop_order(depth: u32, push_order: i64) -> i64 {
        Self::exit_order(depth, -push_order)
    }

    pub(super) const fn overlay_order(depth: u32) -> i64 {
        Self::exit_order(depth, 2)
    }
}
