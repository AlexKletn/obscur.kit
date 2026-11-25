//! The interaction pseudo-states the cascade matches against:
//! [`PseudoStateProjection`].

use obscur_identity::NodeId;
use obscur_interaction::hover::HoverChain;
use obscur_style::selector::PseudoState;
use obscur_tree::tree::RetainedTree;

use super::AncestorChain;
use crate::orchestration::PseudoStates;

#[doc = include_str!("../../docs/api/runtime/pseudo_state_projection.doc/PseudoStateProjection.md")]
pub(crate) struct PseudoStateProjection<'a> {
    /// The tree each press's ancestor chain is walked over.
    tree: &'a RetainedTree,
    /// The chain the pointer is currently over, topmost-first.
    hover: &'a HoverChain,
    /// The focused node, if any.
    focused: Option<NodeId>,
    /// The target of every outstanding press — a `Down` with no matching `Up` or
    /// `Cancel` yet — one per pointer holding one.
    pressed: &'a [NodeId],
    #[doc = include_str!("../../docs/api/runtime/pseudo_state_projection.doc/focus_visible.md")]
    focus_visible: bool,
}

impl<'a> PseudoStateProjection<'a> {
    /// Bundles the live input state for one recomputation.
    pub(crate) const fn new(
        tree: &'a RetainedTree,
        hover: &'a HoverChain,
        focused: Option<NodeId>,
        pressed: &'a [NodeId],
        focus_visible: bool,
    ) -> Self {
        Self {
            tree,
            hover,
            focused,
            pressed,
            focus_visible,
        }
    }

    /// Builds the per-node [`PseudoStates`] from the current hover chain, focus
    /// and outstanding presses. [`PseudoState::FOCUS_WITHIN`] applies to the focused
    /// node and its logical ancestors, including across portal projections.
    pub(crate) fn project(&self) -> PseudoStates {
        let mut states = PseudoStates::new();
        for &node in self.hover.nodes() {
            states.set(node, PseudoState::HOVER);
        }
        if let Some(focused) = self.focused {
            for node in AncestorChain::of(self.tree, Some(focused)) {
                states.set(node, states.get(node).union(PseudoState::FOCUS_WITHIN));
            }
            let mut merged = states.get(focused).union(PseudoState::FOCUS);
            if self.focus_visible {
                merged = merged.union(PseudoState::FOCUS_VISIBLE);
            }
            states.set(focused, merged);
        }
        // `:active` — while a press is outstanding (a `Down` with no matching
        // `Up` or `Cancel` yet), the pressed node and its element ancestors are
        // active; CSS applies `:active` up the ancestor chain, mirroring the hover
        // chain. Every outstanding press contributes: two fingers holding two
        // buttons light both, and one lifting leaves the other lit. The union is
        // order-independent, so the map's iteration order does not leak into the
        // computed styles.
        for &target in self.pressed {
            for node in AncestorChain::of(self.tree, Some(target)) {
                let merged = states.get(node).union(PseudoState::ACTIVE);
                states.set(node, merged);
            }
        }
        states
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_hovered_focused_or_pressed_projects_an_empty_map() {
        let tree = RetainedTree::new();
        let hover = HoverChain::new();
        let states = PseudoStateProjection::new(&tree, &hover, None, &[], false).project();
        assert_eq!(states.get(NodeId::from_raw(1)), PseudoState::empty());
    }

    #[test]
    fn focus_lands_on_one_node_and_hover_on_the_whole_chain() {
        let tree = RetainedTree::new();
        let (a, b) = (NodeId::from_raw(1), NodeId::from_raw(2));
        let hover = HoverChain::from_nodes(vec![a, b]);
        // Focus is where keys go: `b` is hovered as `a`'s ancestor, but focusing
        // `a` must not focus `b` as well.
        let states = PseudoStateProjection::new(&tree, &hover, Some(a), &[], false).project();
        assert_eq!(
            states.get(a),
            PseudoState::HOVER.union(PseudoState::FOCUS.union(PseudoState::FOCUS_WITHIN))
        );
        assert_eq!(states.get(b), PseudoState::HOVER);
    }

    #[test]
    fn two_outstanding_presses_both_light_active() {
        // A tree that resolves neither id, so each press contributes exactly its
        // own node — the chain walk's head — and the union is what is under test.
        let tree = RetainedTree::new();
        let hover = HoverChain::new();
        let (a, b) = (NodeId::from_raw(3), NodeId::from_raw(4));
        let states = PseudoStateProjection::new(&tree, &hover, None, &[a, b], false).project();
        assert_eq!(states.get(a), PseudoState::ACTIVE);
        assert_eq!(states.get(b), PseudoState::ACTIVE);
    }
}
