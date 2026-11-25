//! The CSS `writing-mode` model: the block-flow / line orientation of a box.

#[doc = include_str!("../../docs/api/model/writing_mode.doc/WritingMode.md")]
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
