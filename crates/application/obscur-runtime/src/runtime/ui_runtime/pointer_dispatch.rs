#![doc = include_str!("pointer_dispatch.doc/_module.md")]

use obscur_geometry::Point;
use obscur_identity::{InternedSymbol, NodeId};
use obscur_interaction::event::UiEvent;
use obscur_interaction::pointer::{PointerEvent, PointerEventKind};

use crate::frame::FrameReason;
use crate::runtime::{
    CommandDispatch, EditableField, HoverUpdate, PointerCompletion, PointerDispatch,
    PointerLandingResolver, PointerRelease, StructuralStates,
};

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("pointer_dispatch.doc/handle_pointer.md")]
    pub fn handle_pointer(&mut self, mut event: PointerEvent) -> PointerDispatch {
        // 0–1. Where the event landed: the host's physical position mapped to
        //      content space, folded through the scroll containers it lies within,
        //      and resolved to a target through capture-else-hit-test. All three
        //      steps and the reasons for them are [`PointerLandingResolver`]'s;
        //      what matters here is that the **screen** point and the
        //      **scroll-folded** point are both live below, because a clip is
        //      tested on screen while a box is tested in unscrolled layout space.
        let landing = PointerLandingResolver::new(
            self.surface,
            &self.tree,
            &self.computed,
            &self.boxes,
            &self.scroll,
            &self.hits,
        )
        .resolve(&event, &mut self.pointers);
        let screen_point = landing.screen;
        event.position = landing.folded;
        let mut target = landing.target;

        // 1b. Pan: a begun drag scrolls its container one-to-one with the finger
        //     and takes the sequence away from the click and the drag-selection
        //     that would otherwise be reading the same moves.
        //
        //     Before hover, not after, because the move that *begins* a pan is
        //     also the move that captures the pointer — and a capture that only
        //     took effect on the next event would let that first move hover and
        //     target the row under the finger, which is the row the gesture has
        //     just decided it is not pressing.
        let mut press_changed = false;
        // A contact that wanders is doing something other than waiting, so the
        // hold is ruled out before the pan is even consulted — the same travel
        // that starts a drag ends a hold, and a context action firing in the
        // middle of a scroll is the failure this prevents.
        if event.kind() == PointerEventKind::Move {
            self.pointers.hold_moved(event.pointer(), screen_point);
        }
        let panning = self.update_pan(&event, screen_point, &mut press_changed);
        if panning && let Some(captor) = self.pointers.capture_target(event.pointer()) {
            target = Some(captor);
        }

        // 2. Hover: the chain this event leaves hovered, and whether `:hover` now
        //    matches differently. Which target that chain is built from is the
        //    device's business as much as the hit test's — see [`HoverUpdate`].
        let hover = HoverUpdate::for_event(&self.tree, &self.hover, &event, target);
        let hover_changed = hover.changed();
        self.hover = hover.into_chain();

        // 3. Focus: only a press moves focus. A press also anchors click
        //    detection on the pressed node.
        let mut focus_changed = false;
        if event.kind() == PointerEventKind::Down {
            focus_changed = self.press_landed(&event, screen_point, target, &mut press_changed);
        }

        // 3a-drag. While a drag is armed, a pointer move re-selects the range from
        //    the press anchor to the current pointer offset over the same field —
        //    live drag-selection (spec §5.11). Not a press/release, so it never
        //    perturbs focus or click detection.
        //    A move the pan consumed is not a selection gesture: the finger is
        //    dragging the list, not sweeping over text.
        if event.kind() == PointerEventKind::Move && !panning {
            self.update_text_drag(event.position());
        }

        // 3a-dnd. A data drag ([`begin_drag`]) consumes pointer input while it is
        //    in flight: the move tracks it across drop targets, the release drops
        //    or cancels it. It returns early because a drag is not a click — a
        //    release that ends a drag must not also activate whatever it landed
        //    on, which is the one behaviour a host would notice if this merely ran
        //    alongside. Hover and focus are equally suppressed: mid-drag, the
        //    pointer is carrying something, not pointing at things.
        if self.drag.is_active() {
            return self.drag_in_flight(&event, target);
        }

        // 3b. Click: a release on the *same* node the press landed on is a click
        //    on that node — it **activates** that control. Activation dispatches
        //    the node's `on-click` command if one is bound (bubbling up its
        //    logical-element ancestors, first match wins), else its `on-change`
        //    command — so a checkbox/input bound to `on-change` fires on click
        //    just as a button bound to `on-click` does. The resolved command is
        //    reported back; the press anchor is cleared on every release.
        //
        // §2.3's `current_target`: the node whose binding resolved `command` —
        // an ancestor of `target` whenever the click bubbled to find one.
        let (command, current_target) = if event.kind() == PointerEventKind::Up {
            self.release_landed(&event, screen_point, target, &mut press_changed)
        } else {
            (None, None)
        };

        // 3c. Cancel: the system took the sequence away — a gesture claimed it, the
        //    window lost the contact, the device went away.
        if event.kind() == PointerEventKind::Cancel && self.cancel_landed(&event) {
            press_changed = true;
        }

        // 4. Recompute the interaction pseudo-states from the new hover/focus.
        if hover_changed || focus_changed || press_changed {
            self.recompute_pseudo_states();
            // 5. Schedule a restyle for the next frame.
            self.request_frame(FrameReason::Input);
        }

        PointerDispatch::new(
            target,
            hover_changed,
            focus_changed,
            command,
            current_target,
            event.trust(),
        )
    }

    #[doc = include_str!("pointer_dispatch.doc/press_landed.md")]
    fn press_landed(
        &mut self,
        event: &PointerEvent,
        screen_point: Point,
        target: Option<NodeId>,
        press_changed: &mut bool,
    ) -> bool {
        let mut focus_changed = false;
        // A disabled control is not a focus / press / editing target: a press
        // that lands inside one is inert (it seats no focus, no caret, and no
        // press anchor), so typing can't reach it and its release activates
        // nothing. Crucially it also does NOT blur whatever was focused —
        // clicking a disabled control is not a blur — so focus, the editing
        // session, and the press anchor simply stay wherever they were. A press
        // on empty space (a target that is None, not merely disabled) still
        // blurs, exactly as before.
        let disabled_press =
            target.is_some_and(|t| StructuralStates::of(&self.tree).is_within_disabled(t));
        if !disabled_press {
            let interactive_target = target;
            if self.focused != interactive_target {
                self.focused = interactive_target;
                focus_changed = true;
                // Focus moved: begin/clear the runtime-owned editing session so
                // the focused editable's text/caret/selection is live for keys
                // (and step-2 rendering).
                self.sync_editing_to_focus();
                // §6.15 "Focused inputs may be scrolled into view". A field
                // half off the edge of its scroller — or behind the on-screen
                // keyboard, which is the case that matters on a device with no
                // hardware one — is brought into the visible region. A field
                // already fully visible does not move.
                if let Some(field) =
                    interactive_target.and_then(|t| EditableField::editable_ancestor(&self.tree, t))
                {
                    self.scroll_into_view(field);
                }
            }
            // Flattened deliberately: "no press outstanding" and "a press on
            // empty space" are the same *styling* state (nothing is `:active`),
            // which is what `press_changed` schedules a restyle for.
            // Records the press and resolves its click count (single /
            // double / triple) from this pointer's *own* previous press, so a
            // double click can select the word and a triple click the line
            // under the pointer (spec §5.11). The reported previous target is
            // what says whether the `:active` styling state changed.
            let replaced = self.pointers.begin_press(
                event.pointer(),
                interactive_target,
                event.position(),
                event.timestamp(),
            );
            if replaced != interactive_target {
                *press_changed = true;
            }
            // Click → caret / word / line: after `sync_editing_to_focus` has
            // (re)seated the session for the clicked editable, a single click
            // places the caret and arms a potential drag-selection; a double
            // click selects the word and a triple click the line under the
            // point. A press on a non-editable target clears any armed drag and
            // leaves the session untouched.
            self.begin_text_pointer(interactive_target, event.position());
        }
        // 3a-arm. A direct contact arms what it speculatively started: a pan
        //    for the user-scrollable container it landed in, and the hold that
        //    fires if the finger stays put on something that binds a long
        //    press. Arming is not panning and not holding — the recognizers
        //    stay `Possible` until the finger travels or waits — so a tap here
        //    costs two structs and changes nothing.
        self.arm_gestures(event, screen_point, target);
        focus_changed
    }

    #[doc = include_str!("pointer_dispatch.doc/drag_in_flight.md")]
    fn drag_in_flight(&mut self, event: &PointerEvent, target: Option<NodeId>) -> PointerDispatch {
        match event.kind() {
            PointerEventKind::Move => {
                if self.drag.pointer_moved(event.position(), target) {
                    self.request_frame(FrameReason::Input);
                }
            }
            PointerEventKind::Up => {
                // The release carries a position, and it is the one that
                // decides where the payload landed — a pointer that moves and
                // lets go inside one event must still drop on what it is over.
                self.drag.pointer_moved(event.position(), target);
                self.finish_drag();
            }
            PointerEventKind::Cancel | PointerEventKind::Leave => {
                if self.drag.cancel() {
                    self.request_frame(FrameReason::Input);
                }
            }
            PointerEventKind::Down | PointerEventKind::Enter => {}
        }
        PointerDispatch::new(target, false, false, None, None, event.trust())
    }

    #[doc = include_str!("pointer_dispatch.doc/release_landed.md")]
    fn release_landed(
        &mut self,
        event: &PointerEvent,
        screen_point: Point,
        target: Option<NodeId>,
        press_changed: &mut bool,
    ) -> (Option<InternedSymbol>, Option<NodeId>) {
        let mut command = None;
        let mut current_target = None;
        // Whether this release is a click, on what, and with what run length
        // — the three conditions and why each of them is one are
        // [`PointerRelease`]'s. It consumes the press either way.
        let release = PointerRelease::take(&mut self.pointers, event, target);
        if let Some(pressed) = release.clicked() {
            if let Some((bound, resolved)) = CommandDispatch::activation_command_for(
                &self.tree,
                pressed,
                EditableField::editable_ancestor(&self.tree, pressed).is_some(),
            ) {
                command = Some(resolved);
                current_target = Some(bound);
            }
            // The activating event travels with the command: §2.4's payload
            // (pointer id, device, buttons, position, pressure) and §2.3's
            // trust reach the handler because this envelope carries them.
            self.activate(pressed, UiEvent::from_pointer(*event));
            // A click on a virtualized row: resolve it to the model index and run
            // the list's row-click handler. This is independent of command
            // activation — the row carries no per-row binding, since one symbol
            // could not distinguish the rows — so the geometry is the only thing
            // that says which row was hit.
            if let Some((list, index)) = self.virtual_index_at(screen_point)
                && self
                    .virtual_lists
                    .fire_row_click(list, index, release.clicks())
            {
                self.request_frame(FrameReason::Input);
            }
        }
        if release.press_ended() {
            *press_changed = true;
        }
        // Hold, pan, capture and drag-selection end together — one call, so
        // the four cannot drift apart. A drag still travelling when the
        // contact lifted comes back to be thrown.
        if let Some(thrown) = self
            .pointers
            .end_pointer(event.pointer(), PointerCompletion::Released)
        {
            self.launch_fling(&thrown, event.timestamp());
        }
        (command, current_target)
    }

    #[doc = include_str!("pointer_dispatch.doc/cancel_landed.md")]
    fn cancel_landed(&mut self, event: &PointerEvent) -> bool {
        // The press is discarded *without* a click: an interrupted gesture must
        // not activate the control the pointer was resting on, which is the one
        // thing separating a cancel from a release. Until touch there was no
        // producer of a `Cancel` at all, so the press simply stayed outstanding
        // and its `:active` stuck.
        let press_changed = self
            .pointers
            .take_press(event.pointer())
            .and_then(|press| press.target)
            .is_some();
        // A cancelled pointer has completed just as surely as a released one,
        // and nothing it was carrying is thrown.
        self.pointers
            .end_pointer(event.pointer(), PointerCompletion::Cancelled);
        press_changed
    }
}
