//! Main-axis distribution of items and free space in a flex container.

/// How a flex container distributes its items and free space along the main
/// axis (spec §4.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum JustifyContent {
    /// Pack items toward the main-axis start (the default).
    #[default]
    FlexStart,
    /// Pack items toward the main-axis end.
    FlexEnd,
    /// Center items along the main axis.
    Center,
    /// Distribute items with the first at the start, the last at the end, and
    /// equal space between adjacent items.
    SpaceBetween,
    /// Distribute items with equal space around each item (half-size gaps at
    /// the container edges).
    SpaceAround,
    /// Distribute items with equal space between items and at the container
    /// edges.
    SpaceEvenly,
}

#[cfg(test)]
mod tests {
    use super::JustifyContent;

    #[test]
    fn default_is_flex_start() {
        assert_eq!(JustifyContent::default(), JustifyContent::FlexStart);
    }

    #[test]
    fn variants_are_distinct() {
        assert_ne!(JustifyContent::SpaceBetween, JustifyContent::SpaceAround);
        assert_ne!(JustifyContent::Center, JustifyContent::SpaceEvenly);
    }
}
