#![doc = include_str!("keyboard.doc/_module.md")]

use obscur_interaction::command::{KeyEvent, LogicalKey, Modifiers, NamedKey};
use obscur_interaction::event::UiEvent;
use obscur_view::view::{EVENT_KEY_DOWN, EVENT_KEY_UP};

use crate::frame::FrameReason;
use crate::runtime::{CommandDispatch, KeyDispatch};

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("keyboard.doc/set_modifiers.md")]
    pub fn set_modifiers(&mut self, modifiers: Modifiers) {
        self.modifiers = modifiers;
    }

    #[doc = include_str!("keyboard.doc/handle_key.md")]
    pub fn handle_key(&mut self, event: KeyEvent) -> KeyDispatch {
        if let Some(modifier) = event.physical().modifier() {
            self.modifiers = if event.pressed() {
                self.modifiers.with(modifier)
            } else {
                self.modifiers.without(modifier)
            };
        }
        // The runtime is authoritative about the chord, so it stamps what it
        // knows onto the envelope rather than trusting whatever the caller
        // filled in. A handler can then read the chord off the event it holds.
        let event = event.with_modifiers(self.modifiers);

        // §2.3 steps 1-3 run FIRST, and on every key — before the editing
        // session sees it, before a shortcut resolves, before activation. That
        // ordering is the whole reason the raw event exists: a `keydown`
        // delivered after the field had already taken the character could
        // report the key but never refuse it.
        let prevented = self.dispatch_raw_key(event);
        // §2.3 step 4, and a release has no default action to suppress anyway.
        let acts = !prevented && event.pressed();

        let Some(target) = self.focused else {
            // No focused node means no node leg — no raw target, no edit, no
            // activation. §2.6's route still ends at the window surface and the
            // application, and a chord aimed at those two links used to be
            // unreachable because this whole body returned here.
            //
            // No frame is requested on this path: with nothing focused and
            // nothing resolving, a keystroke changes nothing to draw, and any
            // handler that did run asked for its own frame.
            let command = acts
                .then(|| self.resolve_and_dispatch_shortcut(event))
                .flatten();
            return KeyDispatch::new(None, command);
        };

        // Whether the key was consumed as a text EDIT. When it was, the editing
        // path already scheduled the appropriate frame via `after_edit` (a
        // compose-only `TextEdit` for a same-size edit, or a full `Input`), so the
        // generic restyle-frame request below must NOT run — overriding it with a
        // full `Input` frame would defeat the per-keystroke compose-only fast path.
        let mut consumed_edit = false;
        let mut command = None;
        if acts {
            // Editing takes precedence for a focused editable: a key that the
            // runtime-owned editing session consumes (a printable char,
            // Backspace/Delete, an arrow/Home/End motion, or a select-all/undo/redo
            // chord) is an EDIT, not a shortcut. When consumed, it is neither
            // resolved as a shortcut nor treated as activation — a bare 'a' in a
            // field edits the field.
            if self.try_text_edit(target, &event) {
                consumed_edit = true;
            } else {
                command = self.resolve_and_dispatch_shortcut(event);
                if is_activation_key(event.logical()) && !event.is_repeat() {
                    // Auto-repeat does not re-activate. Holding Space on a
                    // button fires at the platform's repeat rate for as long as
                    // the finger is down, and a control that ran its command
                    // thirty times a second because a key was resting on it is
                    // a defect, not a feature. A held arrow still moves the
                    // caret, because that is the editing path above, not this.
                    self.activate(target, UiEvent::from_key(event));
                }
            }
        }

        if !consumed_edit {
            self.request_frame(FrameReason::Input);
        }
        KeyDispatch::new(Some(target), command)
    }

    #[doc = include_str!("keyboard.doc/dispatch_raw_key.md")]
    fn dispatch_raw_key(&mut self, event: KeyEvent) -> bool {
        let name = if event.pressed() {
            EVENT_KEY_DOWN
        } else {
            EVENT_KEY_UP
        };
        let Some(target) = self.focused else {
            return false;
        };
        let Some((node, command)) = CommandDispatch::binding_for(&self.tree, target, name) else {
            return false;
        };
        self.dispatch_command_outcome(Some(node), command, UiEvent::from_key(event))
            .default_prevented
    }

    #[doc = include_str!("keyboard.doc/resolve_and_dispatch_shortcut.md")]
    fn resolve_and_dispatch_shortcut(
        &mut self,
        event: KeyEvent,
    ) -> Option<obscur_identity::InternedSymbol> {
        let resolved = event
            .logical()
            .and_then(|key| self.routing.command_router().resolve(key, self.modifiers))?;
        // The key travels with the command, so a shortcut handler can read which
        // chord fired it (§2.6's "logical key/text" layer) rather than only that
        // *something* did.
        self.dispatch_command_event(self.focused, resolved, UiEvent::from_key(event));
        Some(resolved)
    }
}

#[doc = include_str!("keyboard.doc/is_activation_key.md")]
fn is_activation_key(key: Option<LogicalKey>) -> bool {
    matches!(
        key,
        Some(LogicalKey::Named(NamedKey::Enter | NamedKey::Space) | LogicalKey::Character(' '))
    )
}
