//! The runtime-owned per-node interaction state source: [`PseudoStates`].

use std::collections::HashMap;

use obscur_identity::NodeId;
use obscur_style::selector::PseudoState;

/// The per-node interaction [`PseudoState`] the runtime owns and the selector
/// matcher reads, keyed by [`NodeId`].
///
/// Interaction-driven pseudo-classes (`:hover`, `:focus`, `:active`, …) can only
/// match when *something* tells the matcher which nodes currently carry those
/// states. `obscur-style` deliberately does not decide that — it only consumes a
/// [`PseudoState`] bitset per element (via
/// [`MatchElement::pseudo_state`](obscur_style::selector::MatchElement::pseudo_state)).
/// `PseudoStates` is that source: the runtime maintains it as input events are
/// dispatched, and [`TreeElement`](super::TreeElement) reads it during matching so
/// authored `:hover`/`:focus` rules finally take effect.
///
/// A node with no recorded state resolves to [`PseudoState::empty`], so a fresh
/// (or fully cleared) map means "no interaction state anywhere" — exactly the
/// pre-Phase-3 behaviour. The map is a plain owned value: cheap to clone and to
/// reset between interaction epochs.
#[derive(Debug, Clone, Default)]
pub struct PseudoStates {
    /// The recorded interaction state for each node that has any non-empty state.
    states: HashMap<NodeId, PseudoState>,
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

    /// Records `state` as `node`'s interaction state, replacing any previous
    /// entry.
    ///
    /// Setting [`PseudoState::empty`] removes the node's entry, keeping the map
    /// canonical: an absent node and a node explicitly set to empty are
    /// indistinguishable to [`get`](Self::get).
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
