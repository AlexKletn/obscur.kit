//! The runtime-owned per-node interaction state source: [`PseudoStates`].

use obscur_identity::{FastMap, NodeId};
use obscur_style::selector::PseudoState;
use obscur_tree::tree::RetainedTree;

#[doc = include_str!("../../docs/api/orchestration/pseudo_states.doc/PseudoStates.md")]
#[derive(Debug, Clone, Default)]
pub struct PseudoStates {
    /// The recorded interaction state for each node that has any non-empty state.
    states: FastMap<NodeId, PseudoState>,
}

impl PseudoStates {
    /// Creates an empty source; every node then reports [`PseudoState::empty`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the interaction state recorded for `node`, or
    /// [`PseudoState::empty`] when the node has no entry.
    #[must_use]
    pub fn get(&self, node: NodeId) -> PseudoState {
        self.states
            .get(&node)
            .copied()
            .unwrap_or_else(PseudoState::empty)
    }

    #[doc = include_str!("../../docs/api/orchestration/pseudo_states.doc/set.md")]
    pub fn set(&mut self, node: NodeId, state: PseudoState) {
        if state.is_empty() {
            self.states.remove(&node);
        } else {
            self.states.insert(node, state);
        }
    }

    /// Clears every recorded interaction state; afterwards every node reports
    /// [`PseudoState::empty`].
    pub fn clear(&mut self) {
        self.states.clear();
    }

    #[doc = include_str!("../../docs/api/orchestration/pseudo_states.doc/retain_live.md")]
    pub fn retain_live(&mut self, tree: &RetainedTree) {
        self.states.retain(|node, _| tree.contains(*node));
    }

    #[doc = include_str!("../../docs/api/orchestration/pseudo_states.doc/iter.md")]
    pub fn iter(&self) -> impl Iterator<Item = (NodeId, PseudoState)> + '_ {
        self.states.iter().map(|(&node, &state)| (node, state))
    }

    /// Returns whether no node carries any interaction state.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    /// Returns the number of nodes carrying a non-empty interaction state.
    #[must_use]
    pub fn len(&self) -> usize {
        self.states.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(raw: u64) -> NodeId {
        NodeId::from_raw(raw)
    }

    #[test]
    fn absent_node_defaults_to_empty() {
        let states = PseudoStates::new();
        assert_eq!(states.get(node(1)), PseudoState::empty());
        assert!(states.is_empty());
        assert_eq!(states.len(), 0);
    }

    #[test]
    fn set_and_get_round_trips() {
        let mut states = PseudoStates::new();
        states.set(node(7), PseudoState::HOVER);
        assert_eq!(states.get(node(7)), PseudoState::HOVER);
        assert!(!states.is_empty());
        assert_eq!(states.len(), 1);
        // An unrelated node is still empty.
        assert_eq!(states.get(node(8)), PseudoState::empty());
    }

    #[test]
    fn set_empty_removes_the_entry() {
        let mut states = PseudoStates::new();
        states.set(node(3), PseudoState::HOVER);
        assert_eq!(states.len(), 1);
        // Setting empty canonicalises to "no entry".
        states.set(node(3), PseudoState::empty());
        assert_eq!(states.get(node(3)), PseudoState::empty());
        assert!(states.is_empty());
    }

    #[test]
    fn set_replaces_previous_state() {
        let mut states = PseudoStates::new();
        states.set(node(4), PseudoState::HOVER);
        states.set(node(4), PseudoState::HOVER.insert(PseudoState::FOCUS));
        let s = states.get(node(4));
        assert!(s.contains(PseudoState::HOVER));
        assert!(s.contains(PseudoState::FOCUS));
    }

    #[test]
    fn clear_resets_every_node() {
        let mut states = PseudoStates::new();
        states.set(node(1), PseudoState::HOVER);
        states.set(node(2), PseudoState::FOCUS);
        states.clear();
        assert!(states.is_empty());
        assert_eq!(states.get(node(1)), PseudoState::empty());
        assert_eq!(states.get(node(2)), PseudoState::empty());
    }
}
