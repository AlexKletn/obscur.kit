#[doc = include_str!("../../docs/api/document/ui_ir_node.doc/UiIrNodeKind.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiIrNodeKind {
    /// A raw element identified by tag.
    Element,
    /// A text leaf carrying literal content.
    Text,
    /// An instance of a named component.
    Component,
    /// A transparent grouping of children with no element of its own.
    Fragment,
    /// A node rendered conditionally.
    Conditional,
    /// A node repeated over a collection.
    Loop,
}

impl UiIrNodeKind {
    /// Returns whether nodes of this kind can hold children — every kind except
    /// [`Text`](UiIrNodeKind::Text).
    #[must_use]
    pub const fn is_container(self) -> bool {
        matches!(
            self,
            Self::Element | Self::Component | Self::Fragment | Self::Conditional | Self::Loop
        )
    }
}
