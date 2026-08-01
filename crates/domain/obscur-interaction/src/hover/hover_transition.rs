#![doc = include_str!("hover_transition.doc/_module.md")]

use obscur_identity::NodeId;

use super::HoverChain;

#[doc = include_str!("hover_transition.doc/HoverTransition.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoverTransition {
    entered: Vec<NodeId>,
    left: Vec<NodeId>,
}

impl HoverTransition {
    #[doc = include_str!("hover_transition.doc/between.md")]
    #[must_use]
    pub fn between(old: &HoverChain, new: &HoverChain) -> Self {
        let entered = new
            .nodes()
            .iter()
            .filter(|node| !old.contains(**node))
            .copied()
            .collect();
        let left = old
            .nodes()
            .iter()
            .filter(|node| !new.contains(**node))
            .copied()
            .collect();
        Self { entered, left }
    }

    #[doc = include_str!("hover_transition.doc/entered.md")]
    #[must_use]
    pub fn entered(&self) -> &[NodeId] {
        &self.entered
    }

    #[doc = include_str!("hover_transition.doc/left.md")]
    #[must_use]
    pub fn left(&self) -> &[NodeId] {
        &self.left
    }

    #[doc = include_str!("hover_transition.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entered.is_empty() && self.left.is_empty()
    }

    #[doc = include_str!("hover_transition.doc/has_changes.md")]
    #[must_use]
    pub fn has_changes(&self) -> bool {
        !self.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(raw: u64) -> NodeId {
        NodeId::from_raw(raw)
    }

    #[test]
    fn unchanged_hover_is_empty() {
        let chain = HoverChain::from_nodes(vec![node(1), node(2)]);
        let t = HoverTransition::between(&chain, &chain);
        assert!(t.is_empty());
        assert!(!t.has_changes());
        assert!(t.entered().is_empty());
        assert!(t.left().is_empty());
    }

    #[test]
    fn entered_preserves_new_order() {
        let old = HoverChain::from_nodes(vec![node(3)]);
        let new = HoverChain::from_nodes(vec![node(1), node(2), node(3)]);
        let t = HoverTransition::between(&old, &new);
        assert_eq!(t.entered(), &[node(1), node(2)]);
        assert!(t.left().is_empty());
        assert!(t.has_changes());
    }

    #[test]
    fn left_preserves_old_order() {
        let old = HoverChain::from_nodes(vec![node(1), node(2), node(3)]);
        let new = HoverChain::from_nodes(vec![node(3)]);
        let t = HoverTransition::between(&old, &new);
        assert_eq!(t.left(), &[node(1), node(2)]);
        assert!(t.entered().is_empty());
    }

    #[test]
    fn simultaneous_enter_and_leave() {
        let old = HoverChain::from_nodes(vec![node(1), node(5)]);
        let new = HoverChain::from_nodes(vec![node(2), node(5)]);
        let t = HoverTransition::between(&old, &new);
        assert_eq!(t.entered(), &[node(2)]);
        assert_eq!(t.left(), &[node(1)]);
    }
}
