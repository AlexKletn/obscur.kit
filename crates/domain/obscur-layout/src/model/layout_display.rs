//! The formatting context a node establishes for its children.

/// The formatting context / display mode of a layout node (spec §4.1).
///
/// Phase-1 supports only the flow (`Block`) and `Flex` formatting contexts,
/// plus `None` for nodes that are laid out away (generate no box). Additional
/// contexts such as grid, table, or positioning are introduced in later phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LayoutDisplay {
    /// Normal flow: the node participates in block layout (the default).
    #[default]
    Block,
    /// Flex layout: the node establishes a flex formatting context.
    Flex,
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
    }
}
