#![doc = include_str!("keyboard.doc/_module.md")]

use obscur_interaction::command::{KeyEvent, LogicalKey, NamedKey};
use obscur_interaction::event::UiEvent;

use crate::frame::FrameReason;
use crate::runtime::KeyDispatch;

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("keyboard.doc/handle_key.md")]
    pub fn handle_key(&mut self, event: KeyEvent) -> KeyDispatch {
        // 1. Fold a physical modifier transition into the held modifier set.
        if let Some(modifier) = event.physical().modifier() {
            self.modifiers = if event.pressed() {
                self.modifiers.with(modifier)
            } else {
                self.modifiers.without(modifier)
            };
        }

        // 2. Route to the focused node; with no focus the key goes nowhere.
        let Some(target) = self.focused else {
            return KeyDispatch::unhandled();
        };

        // 3. On a press, resolve a bound command from the produced logical key +
        //    held modifiers and DISPATCH it, so a keyboard shortcut runs the same
        //    handler a pointer click would (a release never triggers a shortcut).
        // Whether the key was consumed as a text EDIT. When it was, the editing
        // path already scheduled the appropriate frame via `after_edit` (a
        // compose-only `TextEdit` for a same-size edit, or a full `Input`), so the
        // generic restyle-frame request below must NOT run — overriding it with a
        // full `Input` frame would defeat the per-keystroke compose-only fast path.
        let mut consumed_edit = false;
        let command = if event.pressed() {
            // 3a. Editing takes precedence for a focused editable: a key that the
            //     runtime-owned editing session consumes (a printable char,
            //     Backspace/Delete, an arrow/Home/End motion, or a
            //     select-all/undo/redo chord) is an EDIT, not a shortcut. When
            //     consumed, it is neither resolved as a shortcut nor treated as
            //     activation — a bare 'a' in a field edits the field.
            if self.try_text_edit(target, &event) {
                consumed_edit = true;
                None
            } else {
                // Not text: resolve a bound command from the produced logical key
                // + held modifiers and DISPATCH it, so a keyboard shortcut runs
                // the same handler a pointer click would.
                let resolved = event
                    .logical()
                    .and_then(|key| self.routing.command_router().resolve(key, self.modifiers));
                if let Some(command) = resolved {
                    // The key travels with the command, so a shortcut handler can
                    // read which chord fired it (§2.6's "logical key/text" layer)
                    // rather than only that *something* did. `handle_key` is the
                    // platform adapter's door — the OS key event is on the other
                    // side of it and there is no key-injection path — so the
                    // envelope is trusted here, at the boundary, exactly as
                    // `PointerEvent::as_trusted` is called in the winit converter.
                    self.dispatch_command_event(
                        self.focused,
                        command,
                        UiEvent::from_key(event).as_trusted(),
                    );
                }
                // 3b. Enter/Space activates the focused control regardless of any
                //     bound shortcut: a button fires its click command, a checkbox
                //     its change command. Both the shortcut above and this
                //     activation can fire on the same keypress.
                if is_activation_key(event.logical()) {
                    self.activate(target, UiEvent::from_key(event).as_trusted());
                }
                resolved
            }
        } else {
            None
        };

        // 4. The key reached a focused node: schedule a restyle for the next
        //    frame, mirroring the pointer path's request on a change — UNLESS the
        //    key was consumed as a text edit, which already scheduled its own
        //    (possibly compose-only) frame via `after_edit`.
        if !consumed_edit {
            self.request_frame(FrameReason::Input);
        }
        KeyDispatch::new(Some(target), command)
    }
}

#[doc = include_str!("keyboard.doc/is_activation_key.md")]
fn is_activation_key(key: Option<LogicalKey>) -> bool {
    matches!(
        key,
        Some(LogicalKey::Named(NamedKey::Enter | NamedKey::Space) | LogicalKey::Character(' '))
    )
}
