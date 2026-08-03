#![doc = include_str!("focus.doc/_module.md")]

use obscur_identity::{InternedSymbol, NodeId};
use obscur_interaction::event::UiEvent;
use obscur_view::view::{EVENT_BLUR, EVENT_FOCUS};

use crate::runtime::{CommandDispatch, EditableField};

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("focus.doc/move_focus_to.md")]
    pub(super) fn move_focus_to(&mut self, next: Option<NodeId>, cause: UiEvent) -> bool {
        if self.focused == next {
            return false;
        }
        let blurred = self.focused;
        self.focused = next;
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
        self.dispatch_focus_change(blurred, next, cause);
        true
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
