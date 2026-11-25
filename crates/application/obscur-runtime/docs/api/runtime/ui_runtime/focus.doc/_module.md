The one place focus moves.

This module exists because of a claim that is otherwise unenforceable: **every**
focus change emits §2.3's `blur` / `focus` pair. That claim cannot be kept by
remembering to dispatch at each site that assigns focus — it is kept by there
being exactly one such site. [`move_focus_to`](UiRuntime::move_focus_to) owns
the field, and every consequence of moving it hangs off that one call:

1. the assignment;
2. the editing session, re-seated for the newly focused editable — which is also
   what commits the outgoing field's `change`;
3. §6.15's scroll-into-view for a focused field;
4. `blur` on the node that lost it, then `focus` on the node that gained it.

Four things move focus now, and all four go through that one door: the pointer
press, Tab, a programmatic [`focus_node`](UiRuntime::focus_node), and the modal
trap — which both takes focus as it opens and gives it back as it closes.

# The trap, and why it is declared rather than called

§2.5's *modal focus traps* and *restore focus* are one mechanism here, and its
lifetime is the dialog element's rather than a pair of calls. An element carrying
[`FOCUS_TRAP`](obscur_interaction::focus::FOCUS_TRAP) traps focus for as long as
it is committed; `reconcile_focus_traps` opens and closes the scope from the
difference between the traps this frame declares and the ones the last one did.
That is what makes the trap impossible to leak: a dialog removed by a re-render
rather than by its own close button closes its trap all the same, because there
was never a close call to skip.

While a trap is open it is the boundary for all three movers, stated two ways
because they ask two different questions. The press and the programmatic move ask
whether a target is admitted — [`focus_trap_admits`](UiRuntime::focus_trap_admits),
which refuses anything outside the trap and refuses `None` with it, since "focus
nothing" is how a press beside the dialog would otherwise blur the field inside
it. Tab does not ask: its ring is *built* from the trap, so there is no outside
for it to be refused from. What is *not* here:
`FocusTree`'s registration model answers a question nothing asks (the ring is
computed from the retained tree on demand), `tabindex` has no authorable
spelling, and spatial navigation has no runtime path — see the focus row of
`docs/api/12-reality.md`.

# Order, and why it is this one

The move is **seated before either event is dispatched**. A handler that asks
what has focus therefore reads the answer its own event describes, not the one
from before it — the same rule the hover crossings follow. `blur` comes before
`focus` so that a teardown on the outgoing node is never interleaved with the
incoming node's setup, and the outgoing field's `change` comes before both,
which is the order HTML fires them in.

# Neither event is a gate

Both are notifications. Focus has already moved by the time either is
dispatched, so `prevent_default` has nothing to suppress and its outcome is
dropped rather than read. The refusal that *does* exist is one event earlier:
`pointerdown` is dispatched before the press moves focus, and refusing it
suppresses the move. Making these cancellable would mean unwinding a move whose
consequences — the session, the `change`, the scroll — have already run.
