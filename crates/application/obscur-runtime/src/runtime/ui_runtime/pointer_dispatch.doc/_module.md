The pointer event's journey through the runtime: where it landed, what it
hovered, what it pressed, what it released, and what that changed.

Almost nothing here decides anything. Every question the dispatch asks is
already an entity in its own file — [`PointerLandingResolver`] maps the host's
point and resolves the target through capture, [`HoverUpdate`] diffs the hover
chain, [`GestureArming`] and [`PointerSessions`] own the pan/hold/fling
recognizers, [`PointerRelease`] decides whether a release is a click and with
what run length, [`CommandDispatch`] resolves the activation binding,
[`StructuralStates`] answers whether the target is inside a disabled subtree.
What is left after them is the **order**, and the order is the thing: hover is
built after the pan has had its say because the move that begins a pan is also
the move that captures the pointer; focus moves only on a press; a drag in
flight returns before the click path can also activate what it was dropped on;
the pseudo-state recompute runs once at the end rather than after each of the
five things that can change it.

The four phases that do have independent semantics have their own methods here,
because each is a self-contained answer to "what does this *kind* of event
do?": [`press_landed`](UiRuntime::press_landed) seats focus, the editing
session and the press anchor, [`drag_in_flight`](UiRuntime::drag_in_flight)
routes the event to an in-flight data drag instead,
[`release_landed`](UiRuntime::release_landed) turns a release into a click and
an activation, and [`cancel_landed`](UiRuntime::cancel_landed) discards a press
without one. They are methods rather than free functions because each of them
mutates half the runtime, and they stay in this file because the order they run
in is the only thing that makes them correct.

Focus is the one thing this file no longer decides. `press_landed` calls
[`move_focus_to`](UiRuntime::move_focus_to) — the single door focus moves
through — because the claim that every focus change emits §2.3's `blur` /
`focus` pair is only keepable if there is exactly one place that assigns it. The
order still lives here: only a press moves focus, and only an unrefused one.

Three more methods here dispatch §2.3's raw vocabulary rather than decide
anything: [`dispatch_raw_pointer`](UiRuntime::dispatch_raw_pointer) delivers the
event's own name to the hit target,
[`dispatch_hover_crossings`](UiRuntime::dispatch_hover_crossings) delivers
`pointerenter` / `pointerleave` to the nodes the hover chain crossed — which are
not the hit target — and
[`dispatch_context_menu`](UiRuntime::dispatch_context_menu) delivers
`contextmenu` for a secondary press. All three sit here for the same reason as
the rest: *when* they run is the whole of what they are. The raw event runs
before the runtime's own reaction so a refusal is reachable; the crossings run
after the chain is seated so a handler reads the state its own event produced;
the context request runs between the two, because it is a second reading of the
same press and refusing it must reach the same focus move.

The one decision this file does make about buttons is which of them engage.
[`engages_controls`] is a one-line predicate with a whole paragraph behind it:
every default action below was written when the platform layer delivered nothing
but the left button, so delivering the other four without it would have made a
right click press, activate and long-press whatever it landed on.

[`PointerLandingResolver`]: crate::runtime::PointerLandingResolver
[`HoverUpdate`]: crate::runtime::HoverUpdate
[`GestureArming`]: crate::runtime::GestureArming
[`PointerSessions`]: crate::runtime::PointerSessions
[`PointerRelease`]: crate::runtime::PointerRelease
[`CommandDispatch`]: crate::runtime::CommandDispatch
[`StructuralStates`]: crate::runtime::StructuralStates
