use super::WritingMode;

/// The logical axes whose intrinsic sizes are independent of the box contents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SizeContainment {
    #[default]
    /// The contents participate in sizing on both axes.
    None,
    /// The contents do not contribute to the inline size.
    Inline,
    /// The contents do not contribute to either size.
    Both,
}

impl SizeContainment {
    /// Whether the horizontal axis is size-contained in this writing mode.
    #[must_use]
    pub const fn contains_width(self, mode: WritingMode) -> bool {
        matches!(self, Self::Both) || (matches!(self, Self::Inline) && !mode.is_vertical())
    }

    /// Whether the vertical axis is size-contained in this writing mode.
    #[must_use]
    pub const fn contains_height(self, mode: WritingMode) -> bool {
        matches!(self, Self::Both) || (matches!(self, Self::Inline) && mode.is_vertical())
    }
}
