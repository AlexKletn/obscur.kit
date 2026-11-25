#[doc = include_str!("../../docs/api/tree/tree.doc/TreeNode.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeNode {
    pub(super) id: Option<Box<str>>,
    pub(super) label: Box<str>,
    pub(super) depth: u16,
    pub(super) has_children: bool,
}

impl TreeNode {
    /// Creates a node showing `label` at `depth`, marking whether it
    /// `has_children` (and is therefore expandable).
    #[must_use]
    pub fn new(label: impl Into<Box<str>>, depth: u16, has_children: bool) -> Self {
        Self {
            id: None,
            label: label.into(),
            depth,
            has_children,
        }
    }

    /// Assigns a stable, nonempty ID for expansion persistence. IDs must be unique
    /// within the tree.
    #[must_use]
    pub fn with_id(mut self, id: impl Into<Box<str>>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Returns the node label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the node depth (root nodes are `0`).
    #[must_use]
    pub const fn depth(&self) -> u16 {
        self.depth
    }

    /// Returns whether the node is an expandable parent.
    #[must_use]
    pub const fn has_children(&self) -> bool {
        self.has_children
    }
}
