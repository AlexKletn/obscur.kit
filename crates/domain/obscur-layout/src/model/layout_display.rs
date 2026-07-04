//! The formatting context a node establishes for its children.

/// The formatting context / display mode of a layout node (spec §4.1).
///
/// Supports the flow (`Block`), `Flex`, and `Grid` formatting contexts, plus
/// `None` for nodes that are laid out away (generate no box). Additional
/// contexts such as table or positioning are introduced in later phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LayoutDisplay {
    /// Normal flow: the node participates in block layout (the default).
    #[default]
    Block,
    /// Flex layout: the node establishes a flex formatting context.
    Flex,
    /// Grid layout: the node establishes a grid formatting context (a uniform
    /// fixed-column grid; see
    /// [`GridLayout`](crate::algorithm::GridLayout)).
    Grid,
    /// The node generates no box and is excluded from layout.
    None,
}

impl LayoutDisplay {
    /// Returns `true` if this is [`LayoutDisplay::None`].
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
