//! The [`HoverTransition`] — the enter/leave diff between two hover chains
//! (hover enter/leave).

use obscur_identity::NodeId;

use super::HoverChain;

/// The enter/leave diff between a previous and a new [`HoverChain`] (hover
/// enter/leave).
///
/// Computed by [`between`](Self::between): `entered` holds the nodes newly under
/// the pointer (in the new chain but not the old), `left` holds the nodes no
/// longer under the pointer (in the old chain but not the new). Each list
/// preserves the order of the chain it was drawn from. These drive standard
/// pointer enter/leave semantics: an enter fires for each newly-hovered node, a
/// leave for each exited node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoverTransition {
    entered: Vec<NodeId>,
    left: Vec<NodeId>,
}

impl HoverTransition {
    /// Computes the enter/leave diff from `old` to `new`.
    ///
    /// `entered` = nodes in `new` not present in `old`, in `new`'s order;
    /// `left` = nodes in `old` not present in `new`, in `old`'s order.
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

    /// The newly-hovered nodes, in the new chain's order.
    #[must_use]
    pub fn entered(&self) -> &[NodeId] {
        &self.entered
    }

    /// The exited nodes, in the old chain's order.
    #[must_use]
    pub fn left(&self) -> &[NodeId] {
        &self.left
    }

    /// Returns whether the hover is unchanged (nothing entered or left).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entered.is_empty() && self.left.is_empty()
    }

    /// Returns whether there is any enter or leave to fire.
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
