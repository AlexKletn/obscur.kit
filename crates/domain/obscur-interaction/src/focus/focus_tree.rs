#![doc = include_str!("focus_tree.doc/_module.md")]

use std::collections::BTreeMap;

use obscur_identity::NodeId;

use super::{FocusNode, FocusRequest, FocusResult, TabOrder};

#[doc = include_str!("focus_tree.doc/FocusTree.md")]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FocusTree {
    nodes: BTreeMap<NodeId, FocusNode>,
    focused: Option<NodeId>,
}

impl FocusTree {
    #[doc = include_str!("focus_tree.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
            focused: None,
        }
    }

    #[doc = include_str!("focus_tree.doc/register.md")]
    pub fn register(&mut self, node: FocusNode) {
        self.nodes.insert(node.node(), node);
    }

    #[doc = include_str!("focus_tree.doc/unregister.md")]
    pub fn unregister(&mut self, node: NodeId) {
        self.nodes.remove(&node);
        if self.focused == Some(node) {
            self.focused = None;
        }
    }

    #[doc = include_str!("focus_tree.doc/focused.md")]
    #[must_use]
    pub fn focused(&self) -> Option<NodeId> {
        self.focused
    }

    #[doc = include_str!("focus_tree.doc/get.md")]
    #[must_use]
    pub fn get(&self, node: NodeId) -> Option<&FocusNode> {
        self.nodes.get(&node)
    }

    #[doc = include_str!("focus_tree.doc/contains.md")]
    #[must_use]
    pub fn contains(&self, node: NodeId) -> bool {
        self.nodes.contains_key(&node)
    }

    #[doc = include_str!("focus_tree.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    #[doc = include_str!("focus_tree.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    #[doc = include_str!("focus_tree.doc/clear.md")]
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.focused = None;
    }

    #[doc = include_str!("focus_tree.doc/apply.md")]
    pub fn apply(&mut self, request: FocusRequest) -> FocusResult {
        match request {
            FocusRequest::Node(node) => self.focus_node(node),
            FocusRequest::Clear => {
                if self.focused.is_some() {
                    self.focused = None;
                    FocusResult::Cleared
                } else {
                    FocusResult::NoTarget
                }
            }
            FocusRequest::Next
            | FocusRequest::Previous
            | FocusRequest::First
            | FocusRequest::Last => self.navigate(request),
        }
    }

    fn focus_node(&mut self, node: NodeId) -> FocusResult {
        match self.nodes.get(&node) {
            None => FocusResult::NoTarget,
            Some(entry) if !entry.is_focusable() => FocusResult::Rejected(node),
            Some(_) if self.focused == Some(node) => FocusResult::Unchanged(node),
            Some(_) => {
                self.focused = Some(node);
                FocusResult::Focused(node)
            }
        }
    }

    fn navigate(&mut self, request: FocusRequest) -> FocusResult {
        let order = self.tab_order();
        let target = match request {
            FocusRequest::Next => order.next_after(self.focused, true),
            FocusRequest::Previous => order.previous_before(self.focused, true),
            FocusRequest::First => order.first(),
            FocusRequest::Last => order.last(),
            // The caller (`apply`) only routes tab-navigation variants here.
            FocusRequest::Node(_) | FocusRequest::Clear => None,
        };
        match target {
            Some(node) => {
                self.focused = Some(node);
                FocusResult::Focused(node)
            }
            None => FocusResult::NoTarget,
        }
    }

    fn tab_order(&self) -> TabOrder {
        let nodes: Vec<FocusNode> = self.nodes.values().copied().collect();
        TabOrder::compute(&nodes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(raw: u64) -> NodeId {
        NodeId::from_raw(raw)
    }

    fn focusable(raw: u64, tab_index: i32) -> FocusNode {
        FocusNode::new(node(raw))
            .focusable(true)
            .with_tab_index(tab_index)
    }

    #[test]
    fn node_request_focuses_focusable() {
        let mut tree = FocusTree::new();
        tree.register(focusable(1, 0));
        assert_eq!(
            tree.apply(FocusRequest::node(node(1))),
            FocusResult::Focused(node(1))
        );
        assert_eq!(tree.focused(), Some(node(1)));
    }

    #[test]
    fn node_request_rejects_non_focusable() {
        let mut tree = FocusTree::new();
        tree.register(FocusNode::new(node(1))); // not focusable
        assert_eq!(
            tree.apply(FocusRequest::node(node(1))),
            FocusResult::Rejected(node(1))
        );
        assert_eq!(tree.focused(), None);
    }

    #[test]
    fn node_request_no_target_when_unregistered() {
        let mut tree = FocusTree::new();
        assert_eq!(
            tree.apply(FocusRequest::node(node(9))),
            FocusResult::NoTarget
        );
    }

    #[test]
    fn node_request_unchanged_when_already_focused() {
        let mut tree = FocusTree::new();
        tree.register(focusable(1, 0));
        tree.apply(FocusRequest::node(node(1)));
        assert_eq!(
            tree.apply(FocusRequest::node(node(1))),
            FocusResult::Unchanged(node(1))
        );
    }

    #[test]
    fn clear_clears_and_reports_no_target_when_empty() {
        let mut tree = FocusTree::new();
        assert_eq!(tree.apply(FocusRequest::clear()), FocusResult::NoTarget);
        tree.register(focusable(1, 0));
        tree.apply(FocusRequest::node(node(1)));
        assert_eq!(tree.apply(FocusRequest::clear()), FocusResult::Cleared);
        assert_eq!(tree.focused(), None);
    }

    #[test]
    fn first_last_next_previous_navigate_the_tab_order() {
        let mut tree = FocusTree::new();
        tree.register(focusable(1, 0));
        tree.register(focusable(2, 0));
        tree.register(focusable(3, 0));

        assert_eq!(
            tree.apply(FocusRequest::first()),
            FocusResult::Focused(node(1))
        );
        assert_eq!(
            tree.apply(FocusRequest::next()),
            FocusResult::Focused(node(2))
        );
        assert_eq!(
            tree.apply(FocusRequest::next()),
            FocusResult::Focused(node(3))
        );
        // Wraps around at the end.
        assert_eq!(
            tree.apply(FocusRequest::next()),
            FocusResult::Focused(node(1))
        );
        assert_eq!(
            tree.apply(FocusRequest::last()),
            FocusResult::Focused(node(3))
        );
        assert_eq!(
            tree.apply(FocusRequest::previous()),
            FocusResult::Focused(node(2))
        );
    }

    #[test]
    fn navigation_reports_no_target_without_tab_stops() {
        let mut tree = FocusTree::new();
        tree.register(FocusNode::new(node(1)).focusable(true).with_tab_index(-1));
        assert_eq!(tree.apply(FocusRequest::next()), FocusResult::NoTarget);
        assert_eq!(tree.apply(FocusRequest::first()), FocusResult::NoTarget);
    }

    #[test]
    fn unregistering_focused_clears_focus() {
        let mut tree = FocusTree::new();
        tree.register(focusable(1, 0));
        tree.apply(FocusRequest::node(node(1)));
        assert_eq!(tree.focused(), Some(node(1)));
        tree.unregister(node(1));
        assert_eq!(tree.focused(), None);
        assert!(!tree.contains(node(1)));
    }

    #[test]
    fn bookkeeping_helpers() {
        let mut tree = FocusTree::new();
        assert!(tree.is_empty());
        tree.register(focusable(1, 0));
        assert_eq!(tree.len(), 1);
        assert!(tree.get(node(1)).is_some());
        tree.clear();
        assert!(tree.is_empty());
        assert_eq!(tree.focused(), None);
    }
}
