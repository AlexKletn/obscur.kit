#![doc = include_str!("pseudo_states.doc/_module.md")]

use obscur_identity::{FastMap, NodeId};
use obscur_style::selector::PseudoState;

#[doc = include_str!("pseudo_states.doc/PseudoStates.md")]
#[derive(Debug, Clone, Default)]
pub struct PseudoStates {
    #[doc = include_str!("pseudo_states.doc/states.md")]
    states: FastMap<NodeId, PseudoState>,
}

impl PseudoStates {
    #[doc = include_str!("pseudo_states.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("pseudo_states.doc/get.md")]
    #[must_use]
    pub fn get(&self, node: NodeId) -> PseudoState {
        self.states
            .get(&node)
            .copied()
            .unwrap_or_else(PseudoState::empty)
    }

    #[doc = include_str!("pseudo_states.doc/set.md")]
    pub fn set(&mut self, node: NodeId, state: PseudoState) {
        if state.is_empty() {
            self.states.remove(&node);
        } else {
            self.states.insert(node, state);
        }
    }

    #[doc = include_str!("pseudo_states.doc/clear.md")]
    pub fn clear(&mut self) {
        self.states.clear();
    }

    #[doc = include_str!("pseudo_states.doc/iter.md")]
    pub fn iter(&self) -> impl Iterator<Item = (NodeId, PseudoState)> + '_ {
        self.states.iter().map(|(&node, &state)| (node, state))
    }

    #[doc = include_str!("pseudo_states.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    #[doc = include_str!("pseudo_states.doc/len.md")]
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
