//! Cross-axis alignment of items in a flex container.

/// How a flex container aligns its items along the cross axis (spec §4.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlignItems {
    /// Align items toward the cross-axis start.
    FlexStart,
    /// Align items toward the cross-axis end.
    FlexEnd,
    /// Center items along the cross axis.
    Center,
    /// Stretch items to fill the cross axis (the default).
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
    }
}
