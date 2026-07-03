//! The [`FocusNode`] describing a node's focus properties (spec §7.1 focus).

use obscur_identity::NodeId;

/// A node's focus properties within the focus model (spec §7.1).
///
/// Each participating node contributes a `FocusNode` describing whether it can
/// receive focus, its position in the tab sequence, and whether it is currently
/// disabled. The `tab_index` mirrors HTML `tabindex`: a negative value marks the
/// node focusable programmatically but excluded from tab navigation, `0` places
/// it in natural (registration) order, and a positive value gives it an explicit
/// priority that sorts ahead of the natural-order nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocusNode {
    node: NodeId,
    focusable: bool,
    tab_index: i32,
    disabled: bool,
}

impl FocusNode {
    /// Creates focus properties for `node`: not focusable, `tab_index` `0`, and
    /// not disabled.
    #[must_use]
    pub const fn new(node: NodeId) -> Self {
        Self {
            node,
            focusable: false,
            tab_index: 0,
            disabled: false,
        }
    }

    /// Sets whether this node can receive focus, returning the updated value.
    #[must_use]
    pub const fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    /// Sets this node's tab index (HTML `tabindex` semantics), returning the
    /// updated value.
    #[must_use]
    pub const fn with_tab_index(mut self, tab_index: i32) -> Self {
        self.tab_index = tab_index;
        self
    }

    /// Sets whether this node is disabled, returning the updated value.
    #[must_use]
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Returns the identifier of the node these properties describe.
    #[must_use]
    pub const fn node(&self) -> NodeId {
        self.node
    }

    /// Returns this node's tab index.
    #[must_use]
    pub const fn tab_index(&self) -> i32 {
        self.tab_index
    }

    /// Returns whether this node can receive focus at all: focusable and not
    /// disabled. Programmatic focus (including a negative `tab_index`) is allowed
    /// when this is `true`.
    #[must_use]
    pub const fn is_focusable(&self) -> bool {
        self.focusable && !self.disabled
    }

    /// Returns whether this node participates in tab navigation: focusable, not
    /// disabled, and a non-negative `tab_index`.
    #[must_use]
    pub const fn is_tab_focusable(&self) -> bool {
        self.focusable && !self.disabled && self.tab_index >= 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(raw: u64) -> NodeId {
        NodeId::from_raw(raw)
    }

    #[test]
    fn new_defaults_are_not_focusable() {
        let n = FocusNode::new(node(1));
        assert_eq!(n.node(), node(1));
        assert_eq!(n.tab_index(), 0);
        assert!(!n.is_focusable());
        assert!(!n.is_tab_focusable());
    }

    #[test]
    fn focusable_node_is_focusable_and_tab_focusable() {
        let n = FocusNode::new(node(1)).focusable(true);
        assert!(n.is_focusable());
        assert!(n.is_tab_focusable());
    }

    #[test]
    fn disabled_node_is_not_focusable() {
        let n = FocusNode::new(node(1)).focusable(true).disabled(true);
        assert!(!n.is_focusable());
        assert!(!n.is_tab_focusable());
    }

    #[test]
    fn negative_tab_index_is_focusable_but_not_tab_focusable() {
        let n = FocusNode::new(node(1)).focusable(true).with_tab_index(-1);
        assert!(n.is_focusable());
        assert!(!n.is_tab_focusable());
    }

    #[test]
    fn positive_tab_index_is_tab_focusable() {
        let n = FocusNode::new(node(1)).focusable(true).with_tab_index(5);
        assert_eq!(n.tab_index(), 5);
        assert!(n.is_tab_focusable());
    }
}
