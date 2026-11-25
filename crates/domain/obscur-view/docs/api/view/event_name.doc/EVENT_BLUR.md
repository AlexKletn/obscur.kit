This node has lost focus — `on:blur="…"`.

The other half of [`EVENT_FOCUS`](crate::view::EVENT_FOCUS), which states why
neither half is cancellable, where the refusal that *does* exist lives
(`pointerdown`), and why the binding is resolved through the ancestor chain
here when the DOM resolves its own on the focused element alone.

Dispatched **after** the move, to the node that no longer holds focus — so
`blur` fires first and `focus` second, and a handler that tears down on blur
what its `focus` built is never interleaved with the next node's setup.

A focus change with no node newly focused still fires this one: a press on empty
space clears focus, and the node that had it hears `blur` with no `focus` to
follow.

# `change` comes first

For an editable losing focus, the `change` its blur commits (spec §5.11 — the
committed value, if it differs from the value focus arrived with) is dispatched
**before** this event, which is the order HTML fires the two in. `blur` reports
that focus has left; `change` reports what the field was left holding, and a
handler reading the field's value on blur must see the committed one.
