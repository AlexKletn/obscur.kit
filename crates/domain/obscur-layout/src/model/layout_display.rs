#![doc = include_str!("layout_display.doc/_module.md")]

#[doc = include_str!("layout_display.doc/LayoutDisplay.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LayoutDisplay {
    #[doc = include_str!("layout_display.doc/Block.md")]
    #[default]
    Block,
    #[doc = include_str!("layout_display.doc/Flex.md")]
    Flex,
    #[doc = include_str!("layout_display.doc/Grid.md")]
    Grid,
    #[doc = include_str!("layout_display.doc/Contents.md")]
    Contents,
    #[doc = include_str!("layout_display.doc/None.md")]
    None,
}

impl LayoutDisplay {
    #[doc = include_str!("layout_display.doc/is_none.md")]
    #[must_use]
    pub const fn is_none(self) -> bool {
        matches!(self, Self::None)
    }
}

#[cfg(test)]
mod tests {
    use super::LayoutDisplay;

    #[test]
    fn default_is_block() {
        assert_eq!(LayoutDisplay::default(), LayoutDisplay::Block);
    }

    #[test]
    fn is_none_only_for_none() {
        assert!(LayoutDisplay::None.is_none());
        assert!(!LayoutDisplay::Block.is_none());
        assert!(!LayoutDisplay::Flex.is_none());
        assert!(!LayoutDisplay::Grid.is_none());
    }

    #[test]
    fn grid_is_distinct_and_not_none() {
        assert_ne!(LayoutDisplay::Grid, LayoutDisplay::Block);
        assert_ne!(LayoutDisplay::Grid, LayoutDisplay::Flex);
        assert!(!LayoutDisplay::Grid.is_none());
    }
}
