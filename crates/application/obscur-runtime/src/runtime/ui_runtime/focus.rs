#![doc = include_str!("focus.doc/_module.md")]

use obscur_identity::{InternedSymbol, NodeId};
use obscur_interaction::event::{EventOrigin, UiEvent};
use obscur_view::view::{EVENT_BLUR, EVENT_FOCUS};

use crate::runtime::{CommandDispatch, EditableField, FocusOrder};

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("focus.doc/move_focus_to.md")]
    pub(super) fn move_focus_to(&mut self, next: Option<NodeId>, cause: UiEvent) -> bool {
        if self.focused == next {
            return false;
        }
        let blurred = self.focused;
        self.focused = next;
        // `:focus-visible` is decided here and only here, from the envelope that
        // caused the move: a keyboard move shows the ring, a press does not.
        // Reading it off the cause rather than taking a parameter is what stops
        // the two ever disagreeing.
        self.focus_visible = next.is_some() && cause.origin() == EventOrigin::Keyboard;
        // Focus moved: begin/clear the runtime-owned editing session so the
        // focused editable's text/caret/selection is live for keys (and step-2
        // rendering). This is also what commits the outgoing field's `change`,
        // and it runs before `blur` is dispatched because HTML fires the two in
        // that order — a handler reading the field on blur must see the
        // committed value.
        self.sync_editing_to_focus();
        // §6.15 "Focused inputs may be scrolled into view". A field half off the
        // edge of its scroller — or behind the on-screen keyboard, which is the
        // case that matters on a device with no hardware one — is brought into
        // the visible region. A field already fully visible does not move.
        if let Some(field) =
            next.and_then(|node| EditableField::editable_ancestor(&self.tree, node))
        {
            self.scroll_into_view(field);
        }
        // `:focus` and `:focus-visible` moved with it, so the projection the
        // style phase reads is stale until this runs. It sits in the one door
        // rather than in each caller because that is what makes every mover
        // correct by construction — the pointer path recomputes again at the end
        // of its own dispatch, for the hover and press states this cannot know
        // about, and one extra projection per press is a better trade than a
        // future caller that forgets.
        self.recompute_pseudo_states();
        self.dispatch_focus_change(blurred, next, cause);
        true
    }

    #[doc = include_str!("focus.doc/focus_node.md")]
    pub fn focus_node(&mut self, node: Option<NodeId>) -> bool {
        if node.is_some_and(|id| self.tree.get(id).is_none()) {
            return false;
        }
        self.move_focus_to(node, UiEvent::detached(EventOrigin::Synthetic))
    }

    #[doc = include_str!("focus.doc/is_focus_visible.md")]
    #[must_use]
    pub const fn is_focus_visible(&self) -> bool {
        self.focus_visible
    }

    #[doc = include_str!("focus.doc/move_focus_by_tab.md")]
    pub(super) fn move_focus_by_tab(&mut self, backwards: bool, cause: UiEvent) -> bool {
        // The innermost open scope that traps, if any: a modal is exactly the
        // thing that must not let Tab wander back to the page behind it. A
        // non-trapping scope groups commands without owning the ring, so it is
        // not consulted here — `traps()` is the whole test.
        let trapping = self
            .routing
            .scopes()
            .iter()
            .rev()
            .find(|scope| scope.traps())
            .cloned();
        let order = FocusOrder::of(&self.tree, trapping.as_ref());
        // An empty ring must not blur what is focused. `step` answering `None`
        // means "there is nowhere to go", and a Tab that unfocused the field the
        // user was typing in because the screen holds one control would be worse
        // than a Tab that does nothing.
        let Some(next) = order.step(self.focused, backwards) else {
            return false;
        };
        self.move_focus_to(Some(next), cause)
    }

    #[doc = include_str!("focus.doc/drop_focus_if_removed.md")]
    pub(super) fn drop_focus_if_removed(&mut self) {
        let Some(node) = self.focused else {
            return;
        };
        if self.tree.get(node).is_some() {
            return;
        }
        // No `blur` reaches anyone: `binding_for` starts at a node the tree no
        // longer has, finds nothing, and reports nothing. That is the honest
        // outcome rather than a missing one — the node whose handler would run
        // is gone, and synthesising an ancestor to run it on would be inventing
        // a target the author never bound.
        self.move_focus_to(None, UiEvent::detached(EventOrigin::Synthetic));
    }

    #[doc = include_str!("focus.doc/dispatch_focus_change.md")]
    fn dispatch_focus_change(
        &mut self,
        blurred: Option<NodeId>,
        focused: Option<NodeId>,
        cause: UiEvent,
    ) {
        // Blur before focus, and both after the move is seated. Either half may
        // be absent: focusing the first control blurs nothing, and a press on
        // empty space focuses nothing.
        if let Some(node) = blurred {
            self.dispatch_focus_notification(node, EVENT_BLUR, cause);
        }
        if let Some(node) = focused {
            self.dispatch_focus_notification(node, EVENT_FOCUS, cause);
        }
    }

    #[doc = include_str!("focus.doc/dispatch_focus_notification.md")]
    fn dispatch_focus_notification(&mut self, node: NodeId, name: InternedSymbol, cause: UiEvent) {
        // Resolved through the ancestor chain, the same way `dispatch_raw_key`
        // resolves `keydown` on the same focused node — and NOT the way
        // `dispatch_crossing` resolves a hover crossing.
        //
        // The DOM's `focus` / `blur` do not bubble, and the reason that costs it
        // nothing is that the DOM focuses *elements*. This runtime focuses
        // whatever the press hit, and a press on the glyphs of a field hits the
        // field's text leaf: a rule that fired only for the exact focused node
        // would make `<input on:focus="…">` run or not depending on whether the
        // user clicked the text or the padding beside it. So the binding walks,
        // one node gains focus so the walk delivers exactly once, and
        // [`EVENT_FOCUS`](obscur_view::view::EVENT_FOCUS) states the deviation.
        let Some((node, command)) = CommandDispatch::binding_for(&self.tree, node, name) else {
            return;
        };
        // The outcome is deliberately dropped: focus has already moved, so there
        // is no default action left for `prevent_default` to suppress. The
        // refusal that exists is on `pointerdown`, one event earlier — see
        // [`EVENT_FOCUS`](obscur_view::view::EVENT_FOCUS).
        self.dispatch_command_outcome(Some(node), command, cause);
    }
}
