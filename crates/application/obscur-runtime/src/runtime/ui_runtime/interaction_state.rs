#![doc = include_str!("../../../docs/api/runtime/ui_runtime/interaction_state.doc/_module.md")]

use obscur_identity::NodeId;
use obscur_interaction::{command::Modifiers, hover::HoverChain};

use crate::orchestration::ComputedStyles;
use crate::runtime::{AncestorChain, MotionDriver, PointerLandingResolver, PseudoStateProjection};

use super::UiRuntime;

impl UiRuntime {
    /// Returns the node currently under the pointer (the hover target), if any.
    #[must_use]
    pub fn hovered(&self) -> Option<NodeId> {
        self.hover.topmost()
    }

    /// Returns the node that currently holds focus, if any.
    #[must_use]
    pub fn focused(&self) -> Option<NodeId> {
        self.focused
    }

    /// Returns the modifier keys currently held (folded from physical modifier
    /// key transitions seen by [`handle_key`](Self::handle_key)).
    #[must_use]
    pub const fn held_modifiers(&self) -> Modifiers {
        self.modifiers
    }

    /// Re-hit-tests the stationary cursor after geometry changes and refreshes its interaction states.
    pub(super) fn refresh_hover_at_cursor(&mut self) -> bool {
        let hit = self.cursor_hover_chain(&self.computed);
        self.apply_cursor_hover_chain(hit)
    }

    /// Refreshes hover using candidate frame styles before they are installed.
    pub(super) fn refresh_hover_with_styles(&mut self, computed: &mut ComputedStyles) -> bool {
        let presentation = self.motion.apply_presentation(computed, &self.boxes);
        self.hits
            .update_presentation(&self.tree, computed, &self.boxes, &presentation);
        let hit = self.cursor_hover_chain(computed);
        MotionDriver::restore_presentation(computed, presentation);
        self.apply_cursor_hover_chain(hit)
    }

    /// Hit-tests the uncaptured cursor against the supplied frame styles.
    fn cursor_hover_chain(&self, computed: &ComputedStyles) -> Option<HoverChain> {
        let (pointer, position) = self.hover_pointer?;
        if self
            .pointers
            .capture_target(pointer)
            .is_some_and(|node| self.tree.contains(node))
        {
            return None;
        }
        let target = PointerLandingResolver::new(
            self.surface,
            &self.tree,
            computed,
            &self.boxes,
            &self.scroll,
            &self.hits,
        )
        .hit_chain(position)
        .target()
        .map(|hit| hit.node());
        Some(HoverChain::from_nodes(
            AncestorChain::of(&self.tree, target).into_vec(),
        ))
    }

    /// Updates interaction pseudo-states when cursor hit geometry changes.
    fn apply_cursor_hover_chain(&mut self, hit: Option<HoverChain>) -> bool {
        let Some(next) = hit else {
            return false;
        };
        if self.hover.nodes() == next.nodes() {
            return false;
        }
        self.hover = next;
        self.recompute_pseudo_states();
        true
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/interaction_state.doc/recompute_pseudo_states.md")]
    pub(super) fn recompute_pseudo_states(&mut self) {
        let pressed = self.pointers.pressed_targets();
        self.pseudo = PseudoStateProjection::new(
            &self.tree,
            &self.hover,
            self.focused,
            &pressed,
            self.focus_visible,
        )
        .project();
    }
}
