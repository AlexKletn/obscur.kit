#![doc = include_str!("../../../docs/api/runtime/ui_runtime/pointer_dispatch.doc/_module.md")]

use obscur_geometry::Point;
use obscur_identity::{InternedSymbol, NodeId};
use obscur_interaction::event::UiEvent;
use obscur_interaction::hover::HoverTransition;
use obscur_interaction::pointer::{
    PointerButton, PointerButtons, PointerEvent, PointerEventKind, PointerId,
};
use obscur_view::view::{EVENT_CONTEXT_MENU, EVENT_POINTER_ENTER, EVENT_POINTER_LEAVE};

use crate::frame::FrameReason;
use crate::runtime::{
    CommandDispatch, HoverUpdate, PointerCompletion, PointerDispatch, PointerEventBindings,
    PointerLandingResolver, PointerRelease, StructuralStates,
};

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/pointer_dispatch.doc/handle_pointer.md")]
    pub fn handle_pointer(&mut self, mut event: PointerEvent) -> PointerDispatch {
        if event.pointer() == PointerId::MOUSE {
            self.pointers.mouse_buttons = if event.kind() == PointerEventKind::Cancel {
                PointerButtons::new()
            } else {
                event.buttons()
            };
        }
        self.hover_correction_pending = false;
        self.hover_pointer = (event.device().hovers_by_default()
            && !matches!(
                event.kind(),
                PointerEventKind::Leave | PointerEventKind::Cancel
            )
            && event.position.x.is_finite()
            && event.position.y.is_finite())
        .then_some((event.pointer(), event.position));
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

        if let Some(drag) = self.scrollbar_drags.get(&event.pointer()).copied() {
            if self.pointers.capture_target(event.pointer()) == Some(drag.node)
                && self.tree.contains(drag.node)
            {
                let prevented = self.dispatch_raw_pointer(&event, Some(drag.node));
                if self.pointers.capture_target(event.pointer()) != Some(drag.node)
                    || self.drag.is_active()
                {
                    self.finish_scrollbar_drag(event.pointer());
                } else if event.kind() != PointerEventKind::Up
                    || (engages_controls(&event)
                        && !(event.button().is_none()
                            && event.buttons().contains(PointerButton::Primary)))
                {
                    self.move_scrollbar_drag(&event, screen_point, drag, prevented);
                }
                return PointerDispatch::new(
                    Some(drag.node),
                    false,
                    false,
                    None,
                    None,
                    event.trust(),
                )
                .with_default_prevented(prevented);
            }
            self.finish_scrollbar_drag(event.pointer());
        }
        let thumb = if event.kind() == PointerEventKind::Down
            && engages_controls(&event)
            && (event.button().is_some()
                || event.buttons().is_empty()
                || event.buttons().contains(PointerButton::Primary))
            && !self.drag.is_active()
            && self.pointers.capture_target(event.pointer()).is_none()
        {
            target.and_then(|target| self.horizontal_thumb_at(target, screen_point))
        } else {
            None
        };
        if let Some(thumb) = thumb {
            target = Some(thumb.node);
        }

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
        let mut candidate = event;
        candidate.position = screen_point;
        let gesture_claimed = self.update_touch_gesture(candidate, target, &mut press_changed);
        let panning = !gesture_claimed && self.update_pan(&event, screen_point, &mut press_changed);
        if panning && let Some(captor) = self.pointers.capture_target(event.pointer()) {
            target = Some(captor);
        }

        // 2. Hover: the chain this event leaves hovered, and whether `:hover` now
        //    matches differently. Which target that chain is built from is the
        //    device's business as much as the hit test's — see [`HoverUpdate`].
        let hover = HoverUpdate::for_event(&self.tree, &self.hover, &event, target);
        let hover_changed = hover.changed();
        let (chain, crossed) = hover.into_parts();
        self.hover = chain;

        // 2a. The crossings (§2.3): `pointerenter` / `pointerleave`, one per node
        //     the chain newly holds or newly dropped — which is NOT the node this
        //     event targets. A move from one row to its sibling leaves one row
        //     and enters another while targeting only the second.
        //
        //     After the chain is seated: a handler asked what it just entered
        //     reads the hover state, and reading the state from before its own
        //     event would be a lie. Before the raw event at 2b, matching the
        //     order the DOM fires its crossings in — out/leave, over/enter, then
        //     move — so that an author porting a hover interaction gets the
        //     sequence they already know.
        self.dispatch_hover_crossings(&event, &crossed);

        // 2b. The raw pointer event (§2.3): `pointerdown` / `pointerup` /
        //     `pointermove` / `pointercancel`, dispatched to the hit target
        //     through capture / target / bubble BEFORE the runtime does anything
        //     of its own with this event. That ordering is the whole reason the
        //     raw event exists — a `pointerdown` delivered after focus had
        //     already moved could report the press but never refuse it — and it
        //     is what makes each `prevent_default` below reachable.
        //
        //     Here, and not before 1b: the pan has already decided whether this
        //     contact is scrolling, and if it is, `target` is the capture target
        //     rather than the row under the finger. So the raw event names the
        //     same node hover and focus were computed from, and a refusal cannot
        //     cancel a scroll already under way — deliberate, and stated on
        //     [`EVENT_POINTER_MOVE`](obscur_view::view::EVENT_POINTER_MOVE).
        //
        //     Here, and not after the drag-in-flight return below: a data drag
        //     consumes the runtime's OWN reactions to the pointer, not the
        //     device's report of it. An application drawing its own drag
        //     affordance is exactly when it most wants the moves, and refusing
        //     one costs nothing because the drag has already taken away every
        //     default action a refusal could have suppressed.
        let raw_prevented = self.dispatch_raw_pointer(&event, target);

        // 2c. The context request (§2.3): `contextmenu`, from a secondary press
        //     and from nothing else. After the raw `pointerdown` — it is the
        //     more specific reading of the same press — and before step 3, so
        //     that refusing it reaches the one thing a secondary press goes on
        //     to do.
        //
        //     Dispatched even when `pointerdown` was refused, and the two
        //     refusals are OR-ed rather than short-circuited: a listener
        //     declining the focus move has not declined the *request for a
        //     menu*, and an application that draws one wants to hear the right
        //     click either way.
        let menu_prevented = self.dispatch_context_menu(&event, target);
        let prevented = gesture_claimed || raw_prevented || menu_prevented;

        if let Some(thumb) = thumb
            && !prevented
            && self.tree.contains(thumb.node)
            && self.pointers.capture_target(event.pointer()).is_none()
        {
            self.begin_scrollbar_drag(event.pointer(), thumb);
            if hover_changed {
                self.recompute_pseudo_states();
                self.request_frame(FrameReason::Input);
            }
            return PointerDispatch::new(
                Some(thumb.node),
                hover_changed,
                false,
                None,
                None,
                event.trust(),
            );
        }

        // 3. Focus: only a press moves focus. A press also anchors click
        //    detection on the pressed node.
        //    A refused `pointerdown` performs NONE of that: no focus move, no
        //    caret, no press anchor and no gesture arming. Nothing was begun, so
        //    nothing is left outstanding — and because the anchor is what a later
        //    release is matched against, a refused press is also why no click can
        //    follow it — and, since the pair describes a move, why no `blur` or
        //    `focus` is announced either.
        //    §2.3's `blur` / `focus` themselves are emitted one level down, from
        //    the single door focus moves through, so this ordering is the last
        //    one that matters to them: the refusal above is reachable, and the
        //    notification below reports a move that has already happened.
        // 2c-slider. A slider takes the press before anything else does: a drag
        //    on it is not a text selection, not a pan, and not a click on
        //    whatever the hit test happened to land on inside it. It runs after
        //    the raw `pointerdown`, so refusing that still stops it — and before
        //    focus, so the press still focuses the control it moved.
        if event.kind() == PointerEventKind::Down
            && !prevented
            && let Some(hit) = target
        {
            self.begin_slider_drag(event.pointer(), hit, event.position());
        }
        if event.kind() == PointerEventKind::Move && !prevented {
            self.continue_slider_drag(event.pointer(), event.position());
        }
        if matches!(
            event.kind(),
            PointerEventKind::Up | PointerEventKind::Cancel
        ) {
            // Forgotten on both endings, and unconditionally: a drag the system
            // took away must not outlive the contact that owned it.
            self.end_slider_drag(event.pointer());
        }

        let mut focus_changed = false;
        if event.kind() == PointerEventKind::Down && !prevented {
            focus_changed = self.press_landed(&event, screen_point, target, &mut press_changed);
        }

        // 3a-drag. While a drag is armed, a pointer move re-selects the range from
        //    the press anchor to the current pointer offset over the same field —
        //    live drag-selection (spec §5.11). Not a press/release, so it never
        //    perturbs focus or click detection.
        //    A move the pan consumed is not a selection gesture: the finger is
        //    dragging the list, not sweeping over text.
        //    A refused `pointermove` is not one either — that is the default
        //    action a `pointermove` listener has to refuse.
        if event.kind() == PointerEventKind::Move {
            self.selection_scroll_point = None;
            if !panning
                && !prevented
                && !self.drag.is_active()
                && self.pointers.text_drag().is_some()
            {
                self.update_text_drag(event.position());
                self.arm_selection_autoscroll(screen_point);
            }
        }

        // 3a-dnd. A data drag ([`begin_drag`]) consumes pointer input while it is
        //    in flight: the move tracks it across drop targets, the release drops
        //    or cancels it. It returns early because a drag is not a click — a
        //    release that ends a drag must not also activate whatever it landed
        //    on, which is the one behaviour a host would notice if this merely ran
        //    alongside. Hover and focus are equally suppressed: mid-drag, the
        //    pointer is carrying something, not pointing at things.
        if self.drag.is_active() {
            return self
                .drag_in_flight(&event, screen_point, target)
                .with_default_prevented(raw_prevented || menu_prevented);
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
        //
        // A refused `pointerup` suppresses the click and everything that hangs
        // off it, but NOT the bookkeeping the release owes either way — the
        // release is still consumed, the pointer still completed, capture still
        // released, a travelling pan still thrown. Those are not default actions
        // an application can decline; declining them would leave a stuck
        // `:active`, a capture nothing releases, and a finger that lifted while
        // the runtime still believes it is down.
        //
        // A release that is not the primary button's does none of it, including
        // the bookkeeping — because the matching press did none either. Letting
        // a secondary release through would make it consume, and click, the
        // press the primary button is still holding down.
        let (command, current_target) =
            if event.kind() == PointerEventKind::Up && engages_controls(&event) {
                self.release_landed(&event, screen_point, target, &mut press_changed, prevented)
            } else {
                (None, None)
            };

        // 3c. Cancel: the system took the sequence away — a gesture claimed it, the
        //    window lost the contact, the device went away.
        //    `pointercancel` was dispatched at 2b and its refusal is deliberately
        //    inert: what follows a cancel is not a default action but the
        //    unwinding of state the system has already destroyed. Honouring a
        //    refusal here would reinstate exactly the stuck press this branch was
        //    written to clear.
        if event.kind() == PointerEventKind::Cancel && self.cancel_landed(&event) {
            press_changed = true;
        }

        if hover_changed || focus_changed || press_changed {
            self.recompute_pseudo_states();
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
        .with_default_prevented(raw_prevented || menu_prevented)
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/pointer_dispatch.doc/press_landed.md")]
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
        // A press outside an open focus trap is inert for exactly the same
        // reasons, and by the same rule: while a modal is up, the page behind it
        // is not a place a press can put focus, a caret, or `:active`. Empty
        // space counts as outside — "focus nothing" is how a press beside the
        // dialog would otherwise blur the field inside it, which is the escape a
        // trap exists to close. The gesture arming below is left alone: a trap
        // owns focus, not the scroll wheel of the list behind it.
        let outside_trap = !self.focus_trap_admits(target);
        let inert_press = disabled_press || outside_trap;
        // Which of the two halves below this press earns. A secondary or middle
        // press moves focus and stops there: a right click focuses the field its
        // context menu is about, the way every platform's does, but it presses
        // nothing — no `:active`, no caret, no drag-selection, no armed pan, and
        // no click on release. See [`engages_controls`].
        let engages = engages_controls(event);
        if !inert_press {
            let interactive_target = target;
            // The one door focus moves through, wherever the move came from —
            // the assignment, the editing session it re-seats, §6.15's
            // scroll-into-view and §2.3's `blur` / `focus` pair are all on the
            // other side of it. The press is its only caller today; that is
            // what makes it the only path, not what makes it the only door.
            focus_changed = self.move_focus_to(interactive_target, UiEvent::from_pointer(*event));
            if engages {
                // Flattened deliberately: "no press outstanding" and "a press on
                // empty space" are the same *styling* state (nothing is
                // `:active`), which is what `press_changed` schedules a restyle
                // for.
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
        }
        // 3a-arm. A direct contact arms what it speculatively started: a pan
        //    for the user-scrollable container it landed in, and the hold that
        //    fires if the finger stays put on something that binds a long
        //    press. Arming is not panning and not holding — the recognizers
        //    stay `Possible` until the finger travels or waits — so a tap here
        //    costs two structs and changes nothing.
        //    A non-engaging press arms neither: dragging with the right button
        //    held is not a scroll, and holding it is not a long press.
        if engages {
            self.arm_gestures(event, screen_point, target);
        }
        focus_changed
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/pointer_dispatch.doc/dispatch_context_menu.md")]
    fn dispatch_context_menu(&mut self, event: &PointerEvent, target: Option<NodeId>) -> bool {
        if event.kind() != PointerEventKind::Down
            || event.button() != Some(PointerButton::Secondary)
        {
            return false;
        }
        // No target is no node leg, the same rule the raw names follow: a right
        // click on empty space names nothing to open a menu for.
        let Some(target) = target else {
            return false;
        };
        let Some((node, command)) =
            CommandDispatch::binding_for(&self.tree, target, EVENT_CONTEXT_MENU)
        else {
            return false;
        };
        self.dispatch_command_outcome(Some(node), command, UiEvent::from_pointer(*event))
            .default_prevented
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/pointer_dispatch.doc/dispatch_raw_pointer.md")]
    fn dispatch_raw_pointer(&mut self, event: &PointerEvent, target: Option<NodeId>) -> bool {
        // The crossing KINDS are dispatched under no name here. `pointerenter` /
        // `pointerleave` describe a node the hover chain crossed, not a kind of
        // report, so step 2a emits them from the transition — which already sees
        // the two crossings these kinds cause, an `Enter` entering whatever it
        // landed on and a `Leave` emptying the chain. Naming them here as well
        // would dispatch each surface crossing twice.
        let Some(name) = PointerEventBindings::name_for(event.kind()) else {
            return false;
        };
        // Cost. A move arrives at the pointer's report rate, and resolving a
        // binding means an ancestor walk that allocates. Nothing below runs
        // unless some node in the committed tree actually binds this name: the
        // walk is skipped, not merely empty. Same trade `CommandDispatch::
        // propagate` makes with `has_listeners`.
        if !self.pointer_bindings.binds(name) {
            return false;
        }
        // No target is no node leg. A press on empty space has nothing to
        // capture, target or bubble through; §2.6's window and application links
        // belong to the shortcut path, not to an event that is *about* a node.
        let Some(target) = target else {
            return false;
        };
        let Some((node, command)) = CommandDispatch::binding_for(&self.tree, target, name) else {
            return false;
        };
        self.dispatch_command_outcome(Some(node), command, UiEvent::from_pointer(*event))
            .default_prevented
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/pointer_dispatch.doc/dispatch_hover_crossings.md")]
    fn dispatch_hover_crossings(&mut self, event: &PointerEvent, crossed: &HoverTransition) {
        // Cost, and the same gate `dispatch_raw_pointer` uses. A crossing is
        // already rarer than a move — the lists below are empty unless the chain
        // actually changed — but sweeping down a list crosses two boundaries per
        // row, so an application that binds neither name must still pay nothing.
        let leaving = self.pointer_bindings.binds(EVENT_POINTER_LEAVE);
        let entering = self.pointer_bindings.binds(EVENT_POINTER_ENTER);
        if !leaving && !entering {
            return;
        }
        // Leave before enter, and the old chain innermost-first: a row is left
        // before the container that held it, so the two sequences nest the way a
        // stack unwinds. A handler that tears down on leave what it built on
        // enter is then never interleaved with the next node's setup, even when
        // the pointer crosses straight from one sibling into another.
        if leaving {
            for node in crossed.left() {
                self.dispatch_crossing(*node, EVENT_POINTER_LEAVE, event);
            }
        }
        // Enter outermost-first, which is the reverse of the chain's own order:
        // a container is entered before the row inside it, so a handler opening a
        // panel on entering the container runs before the row that lives in it
        // asks to be highlighted.
        if entering {
            for node in crossed.entered().iter().rev() {
                self.dispatch_crossing(*node, EVENT_POINTER_ENTER, event);
            }
        }
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/pointer_dispatch.doc/dispatch_crossing.md")]
    fn dispatch_crossing(&mut self, node: NodeId, name: InternedSymbol, event: &PointerEvent) {
        // The node's OWN binding, with no ancestor walk — unlike every other
        // event here. The crossing set is already the whole chain, so resolving
        // through ancestors would dispatch a container's binding once per
        // descendant that crossed with it: N deliveries for one boundary. A node
        // that left the tree between frames resolves to nothing and is skipped.
        let Some(command) = self.tree.get(node).and_then(|n| n.command_for_event(name)) else {
            return;
        };
        // The outcome is deliberately dropped: a hover transition has no default
        // action to suppress. The chain is already seated and `:hover` already
        // matches differently — that is not the crossing's default action, it is
        // the crossing. `prevent_default` on one of these is inert, and
        // [`EVENT_POINTER_ENTER`](obscur_view::view::EVENT_POINTER_ENTER) says so.
        self.dispatch_command_outcome(Some(node), command, UiEvent::from_pointer(*event));
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/pointer_dispatch.doc/drag_in_flight.md")]
    fn drag_in_flight(
        &mut self,
        event: &PointerEvent,
        screen_point: Point,
        target: Option<NodeId>,
    ) -> PointerDispatch {
        match event.kind() {
            PointerEventKind::Move => {
                self.arm_drag_autoscroll(screen_point);
                if self.drag.pointer_moved(event.position(), target) {
                    self.request_frame(FrameReason::Input);
                }
            }
            PointerEventKind::Up => {
                self.drag_scroll_point = None;
                // The release carries a position, and it is the one that
                // decides where the payload landed — a pointer that moves and
                // lets go inside one event must still drop on what it is over.
                self.drag.pointer_moved(event.position(), target);
                self.finish_drag();
            }
            PointerEventKind::Cancel | PointerEventKind::Leave => {
                self.drag_scroll_point = None;
                if self.drag.cancel() {
                    self.request_frame(FrameReason::Input);
                }
            }
            PointerEventKind::Down | PointerEventKind::Enter => {}
        }
        PointerDispatch::new(target, false, false, None, None, event.trust())
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/pointer_dispatch.doc/release_landed.md")]
    fn release_landed(
        &mut self,
        event: &PointerEvent,
        screen_point: Point,
        target: Option<NodeId>,
        press_changed: &mut bool,
        prevented: bool,
    ) -> (Option<InternedSymbol>, Option<NodeId>) {
        let mut command = None;
        let mut current_target = None;
        // Whether this release is a click, on what, and with what run length
        // — the three conditions and why each of them is one are
        // [`PointerRelease`]'s. It consumes the press either way.
        let release = PointerRelease::take(&mut self.pointers, event, target);
        if let Some(pressed) = release.clicked().filter(|_| !prevented) {
            if let Some((bound, resolved, _)) =
                self.dispatch_activation(pressed, UiEvent::from_pointer(*event))
            {
                command = Some(resolved);
                current_target = Some(bound);
            }
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

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/pointer_dispatch.doc/cancel_landed.md")]
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

#[doc = include_str!("../../../docs/api/runtime/ui_runtime/pointer_dispatch.doc/engages_controls.md")]
const fn engages_controls(event: &PointerEvent) -> bool {
    matches!(event.button(), None | Some(PointerButton::Primary))
}
