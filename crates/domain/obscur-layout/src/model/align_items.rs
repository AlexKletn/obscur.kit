//! Cross-axis / grid default alignment of items in a container.

#[doc = include_str!("../../docs/api/model/align_items.doc/AlignItems.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlignItems {
    /// Align items toward the cross-axis start (flexbox spelling).
    FlexStart,
    /// Align items toward the cross-axis end (flexbox spelling).
    FlexEnd,
    /// Align items toward the start of the axis (grid / CSS-align spelling).
    Start,
    /// Align items toward the end of the axis (grid / CSS-align spelling).
    End,
    /// Center items along the axis.
    Center,
    /// Aligns the first baseline with the other items in its baseline-sharing group.
    Baseline,
    /// Aligns the last baseline with its baseline-sharing group.
    LastBaseline,
    /// Stretch items to fill the axis (the default).
    #[default]
    Stretch,
}

#[cfg(test)]
mod tests {
    use super::AlignItems;

    #[test]
    fn default_is_stretch() {
        assert_eq!(AlignItems::default(), AlignItems::Stretch);
    }

    #[test]
    fn variants_are_distinct() {
        assert_ne!(AlignItems::FlexStart, AlignItems::FlexEnd);
        assert_ne!(AlignItems::Center, AlignItems::Stretch);
        assert_ne!(AlignItems::Start, AlignItems::End);
        assert_ne!(AlignItems::Start, AlignItems::FlexStart);
    }
}
