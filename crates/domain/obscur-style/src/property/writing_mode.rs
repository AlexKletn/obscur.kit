#![doc = include_str!("writing_mode.doc/_module.md")]

#[doc = include_str!("writing_mode.doc/WritingMode.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WritingMode {
    #[doc = include_str!("writing_mode.doc/HorizontalTb.md")]
    #[default]
    HorizontalTb,
    #[doc = include_str!("writing_mode.doc/VerticalRl.md")]
    VerticalRl,
    #[doc = include_str!("writing_mode.doc/VerticalLr.md")]
    VerticalLr,
}

impl WritingMode {
    #[doc = include_str!("writing_mode.doc/is_vertical.md")]
    #[must_use]
    pub const fn is_vertical(self) -> bool {
        matches!(self, Self::VerticalRl | Self::VerticalLr)
    }
}
