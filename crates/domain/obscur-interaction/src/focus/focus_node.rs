#![doc = include_str!("focus_node.doc/_module.md")]

use obscur_identity::NodeId;

#[doc = include_str!("focus_node.doc/FocusNode.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocusNode {
    node: NodeId,
    focusable: bool,
    tab_index: i32,
    disabled: bool,
}

impl FocusNode {
    #[doc = include_str!("focus_node.doc/new.md")]
    #[must_use]
    pub const fn new(node: NodeId) -> Self {
        Self {
            node,
            focusable: false,
            tab_index: 0,
            disabled: false,
        }
    }

    #[doc = include_str!("focus_node.doc/focusable.md")]
    #[must_use]
    pub const fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    #[doc = include_str!("focus_node.doc/with_tab_index.md")]
    #[must_use]
    pub const fn with_tab_index(mut self, tab_index: i32) -> Self {
        self.tab_index = tab_index;
        self
    }

    #[doc = include_str!("focus_node.doc/disabled.md")]
    #[must_use]
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    #[doc = include_str!("focus_node.doc/node.md")]
    #[must_use]
    pub const fn node(&self) -> NodeId {
        self.node
    }

    #[doc = include_str!("focus_node.doc/tab_index.md")]
    #[must_use]
    pub const fn tab_index(&self) -> i32 {
        self.tab_index
    }

    #[doc = include_str!("focus_node.doc/is_focusable.md")]
    #[must_use]
    pub const fn is_focusable(&self) -> bool {
        self.focusable && !self.disabled
    }

    #[doc = include_str!("focus_node.doc/is_tab_focusable.md")]
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
