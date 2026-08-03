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

Today the only caller is the pointer press. That is the honest state of the
subsystem rather than a design: §2.5 names a focus tree, tab order and
programmatic focus, and none of the three has a runtime path yet — `TabOrder`,
`FocusTree` and `SpatialNavigation` live in the domain tier with no caller here.
When one of them arrives it moves focus through this function, and the pair
fires without that change having to remember to fire it.

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
