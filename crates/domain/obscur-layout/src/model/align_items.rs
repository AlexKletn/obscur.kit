//! Cross-axis / grid default alignment of items in a container.

/// How a container aligns its items along the cross axis (flex, spec §4.1) or a
/// grid item within its cell on one axis (the container default, spec §4.1 grid).
///
/// The `FlexStart` / `FlexEnd` spellings are the flexbox cross-axis values; the
/// `Start` / `End` spellings are the grid (and CSS-align) values. They are kept
/// as distinct variants so flex layout can key off its own pair without a grid
/// value ever perturbing it; a grid formatting context treats `FlexStart` and
/// `Start` (and `FlexEnd` / `End`) as the same physical alignment.
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
