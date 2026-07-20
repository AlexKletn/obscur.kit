//! The CSS `writing-mode` model: the block-flow / line orientation of a box.

/// The block-flow direction a box lays its in-flow content along (`writing-mode`,
/// css-writing-modes-4 §3).
///
/// This is layout's own vocabulary for the style crate's `writing-mode`, carried
/// on [`LayoutStyle`](crate::model::LayoutStyle) so
/// [`FlowLayout`](crate::algorithm::FlowLayout) can stack its in-flow children
/// along the correct physical axis and origin. It decides which physical edge
/// `block-start` names and which way block flow progresses (§6.2):
///
/// * [`HorizontalTb`](Self::HorizontalTb) — the CSS initial. The block axis is
///   vertical: `block-start` is the physical top and children stack top-to-bottom
///   (+y), exactly as before `writing-mode` existed.
/// * [`VerticalRl`](Self::VerticalRl) — the block axis is horizontal and flows
///   right-to-left: `block-start` is the physical right edge and children stack
///   from the right edge leftward.
/// * [`VerticalLr`](Self::VerticalLr) — the block axis is horizontal and flows
///   left-to-right: `block-start` is the physical left edge and children stack
///   from the left edge rightward.
///
/// In both vertical modes the **inline** axis (the line direction) is vertical,
/// running top-to-bottom for the default `ltr` inline base direction.
///
/// # What this does *not* cover
///
/// Only the block-progression geometry of normal flow reads this today. Vertical
/// text shaping / line breaking (the text stack's job) and the vertical-axis
/// reversal of flex and grid layout are out of scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WritingMode {
    /// `horizontal-tb` — the CSS initial. Block flow is top-to-bottom.
    #[default]
    HorizontalTb,
    /// `vertical-rl` — block flow is right-to-left (block-start = physical right).
    VerticalRl,
    /// `vertical-lr` — block flow is left-to-right (block-start = physical left).
    VerticalLr,
}

impl WritingMode {
    /// Whether the **block** axis is horizontal — i.e. one of the two vertical
    /// writing modes, under which normal flow stacks children left/right rather
    /// than top/bottom and the inline axis is the vertical one.
    #[must_use]
    pub const fn is_vertical(self) -> bool {
        matches!(self, Self::VerticalRl | Self::VerticalLr)
    }
}

#[cfg(test)]
mod tests {
    use super::WritingMode;

    #[test]
    fn default_is_horizontal_tb_and_not_vertical() {
        assert_eq!(WritingMode::default(), WritingMode::HorizontalTb);
        assert!(!WritingMode::default().is_vertical());
    }

    #[test]
    fn both_vertical_modes_report_vertical() {
        assert!(WritingMode::VerticalRl.is_vertical());
        assert!(WritingMode::VerticalLr.is_vertical());
    }
}
