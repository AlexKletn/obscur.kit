Dispatches a hit-tested pointer event: updates the hover chain and focus,
recomputes the per-node [`PseudoStates`], and schedules a frame when
anything changed (§3.10, §4.7).

The flow is:

1. **Hit-test** `event.position` against the retained spatial index; the
   topmost hit (highest z-order = innermost element) is the `target`.
2. **Hover** — build the new hover chain as `target` plus its element
   ancestors (topmost-first toward root) and diff it against the old chain
   via [`HoverTransition`]. A non-empty transition marks `hover_changed`.
3. **Focus** — a [`PointerEventKind::Down`] press moves focus to `target`;
   `focus_changed` records whether that differed from the prior focus.
4. **Recompute pseudo-states** — when hover or focus changed, clear the
   interaction map, set [`PseudoState::HOVER`] on every node in the hover
   chain, then merge [`PseudoState::FOCUS`] into the focused node's state.
5. **Schedule** — if anything changed, request a
   [`FrameReason::Input`] frame so the next `run_due_frame`/`render_now`
   restyles with the updated `:hover`/`:focus` state.

It never itself renders; it only mutates interaction state and requests a
frame. Returns a [`PointerDispatch`] describing the target and what moved.

# §2.3 `current_target` and trust

The returned dispatch reports `current_target` — the node whose binding
resolved the command, which is an *ancestor* of `target` whenever the
click bubbled — and `trust`, carried through unchanged from `event`.

Trust is **reported, not enforced**: this method treats an untrusted event
exactly as it treats a trusted one, because §7.6's DevTools agent loop
exists precisely to drive real clicks. The application decides whether a
given action deserves a real gesture.

[`PseudoStates`]: crate::orchestration::PseudoStates
[`HoverTransition`]: obscur_interaction::hover::HoverTransition
[`PseudoState::HOVER`]: obscur_style::selector::PseudoState::HOVER
[`PseudoState::FOCUS`]: obscur_style::selector::PseudoState::FOCUS
