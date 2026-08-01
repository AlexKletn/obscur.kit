#![doc = include_str!("writing_mode.doc/_module.md")]

#[doc = include_str!("writing_mode.doc/WritingMode.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
