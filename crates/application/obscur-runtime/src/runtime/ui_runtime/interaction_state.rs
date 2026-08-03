#![doc = include_str!("interaction_state.doc/_module.md")]

use obscur_identity::NodeId;
use obscur_interaction::command::Modifiers;

use crate::runtime::PseudoStateProjection;

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("interaction_state.doc/hovered.md")]
    #[must_use]
    pub fn hovered(&self) -> Option<NodeId> {
        self.hover.topmost()
    }

    #[doc = include_str!("interaction_state.doc/focused.md")]
    #[must_use]
    pub fn focused(&self) -> Option<NodeId> {
        self.focused
    }

    #[doc = include_str!("interaction_state.doc/held_modifiers.md")]
    #[must_use]
    pub const fn held_modifiers(&self) -> Modifiers {
        self.modifiers
    }

    #[doc = include_str!("interaction_state.doc/recompute_pseudo_states.md")]
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
