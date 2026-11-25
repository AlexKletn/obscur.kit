#![doc = include_str!("../../docs/api/property/writing_mode.doc/_module.md")]

#[doc = include_str!("../../docs/api/property/writing_mode.doc/WritingMode.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WritingMode {
    /// `horizontal-tb` — the CSS initial. Block flow is top-to-bottom (the block
    /// axis is vertical), inline text runs horizontally.
    #[default]
    HorizontalTb,
    /// `vertical-rl` — block flow is right-to-left (the block axis is horizontal,
    /// `block-start` is the physical right edge), inline text runs top-to-bottom.
    VerticalRl,
    /// `vertical-lr` — block flow is left-to-right (the block axis is horizontal,
    /// `block-start` is the physical left edge), inline text runs top-to-bottom.
    VerticalLr,
}

impl WritingMode {
    /// Whether the **block** axis is horizontal — i.e. this is one of the two
    /// vertical writing modes, so normal flow stacks its children left/right rather
    /// than top/bottom and the inline axis is the vertical one.
    #[must_use]
    pub const fn is_vertical(self) -> bool {
        matches!(self, Self::VerticalRl | Self::VerticalLr)
    }
}
