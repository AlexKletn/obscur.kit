Dispatches a hit-tested pointer event: updates the hover chain and focus,
recomputes the per-node [`PseudoStates`], and schedules a frame when
anything changed (§3.10, §4.7).

The flow is:

1. **Hit-test** `event.position` against the retained spatial index; the
   topmost hit (highest z-order = innermost element) is the `target`.
2. **Hover** — build the new hover chain as `target` plus its element
   ancestors (topmost-first toward root) and diff it against the old chain
   via [`HoverTransition`]. A non-empty transition marks `hover_changed`.
3. **The crossings** — `pointerleave` for every node the chain dropped,
   innermost-first, then `pointerenter` for every node it gained,
   outermost-first. These name the nodes a boundary was crossed at, which are
   not `target`: a move from one row to its sibling leaves one and enters the
   other while targeting only the second. A crossing has no default action, so
   nothing here is refusable; `dispatch_hover_crossings` says why the order is
   what it is.
4. **The raw event** — `pointerdown` / `pointerup` / `pointermove` /
   `pointercancel` is dispatched to `target` through §2.3's phases *before* any
   of the runtime's own behaviour below, so a listener that calls
   `prevent_default` can refuse it. `dispatch_raw_pointer` documents what each
   refusal suppresses, what it deliberately does not, and why the unbound case
   costs nothing.
5. **The context request** — `contextmenu`, dispatched to `target` when and only
   when this is a **secondary**-button press. It follows the raw name rather
   than replacing it, and its refusal joins that one:
   `dispatch_context_menu` says what the refusal reaches and why it is not
   behind the raw names' binding gate.
6. **Focus** — a [`PointerEventKind::Down`] press moves focus to `target`;
   `focus_changed` records whether that differed from the prior focus. A refused
   `pointerdown` — or a refused `contextmenu` — skips all of it, so no click can
   follow either.

   Only the **primary** button goes further. A secondary or middle press stops
   at the focus move: no press anchor, no caret, no armed gesture, and no
   activation when it is released. `engages_controls` is that rule, and a
   release that does not engage never reaches `release_landed` at all —
   including the bookkeeping a release normally owes, because the matching press
   never began any.
7. **Recompute pseudo-states** — when hover or focus changed, clear the
   interaction map, set [`PseudoState::HOVER`] on every node in the hover
   chain, then merge [`PseudoState::FOCUS`] into the focused node's state.
8. **Schedule** — if anything changed, request a
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
