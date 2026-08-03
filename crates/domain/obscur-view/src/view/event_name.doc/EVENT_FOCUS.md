This node has gained focus — `on:focus="…"`.

Dispatched **after** the move, to the node that now holds focus. By the time it
runs, the runtime's own `focused()` already answers with this node, the session
for the newly focused editable is already seeded, and any scroll needed to bring
it into view has already happened. A handler asked "what has focus?" therefore
reads the truth rather than the state from before its own event — the same rule
[`EVENT_POINTER_ENTER`](crate::view::EVENT_POINTER_ENTER) follows for the hover
chain.

# `prevent_default` has nothing to suppress here

Deliberate, and the honest finding rather than an invented symmetry. This is a
**notification, not a gate**: focus has already moved when the event is
dispatched, so there is no default action left to decline. Refusing one is
inert.

The refusal that does exist is one event earlier.
[`EVENT_POINTER_DOWN`](crate::view::EVENT_POINTER_DOWN) is dispatched *before*
the press moves focus, and refusing **that** suppresses the move — along with
the caret, the press anchor and the gesture arming. That is where a handler
declines a focus change; there is nothing here to make cancellable, and making
this one cancellable would mean unwinding a move whose consequences (the editing
session, the `change` its blur committed, the scroll) have already run.

# Pairing and order

A move between two nodes fires both halves, and **`blur` first**: the node
losing focus hears about it before the node gaining it does. A move to or from
nothing fires only the half that has a node — focusing the first control fires
`focus` alone, and a press on empty space clears focus and fires `blur` alone.

# The binding bubbles — unlike the DOM's, and on purpose

Resolved by walking the focused node's ancestors, first binding wins, exactly as
[`EVENT_KEY_DOWN`](crate::view::EVENT_KEY_DOWN) is resolved on the same node.

The DOM's `focus` and `blur` do **not** bubble, and the reason that costs the
DOM nothing is that the DOM focuses *elements*. This runtime focuses whatever
the press hit, and a press on a field's glyphs hits the field's **text leaf**.
Resolving on the focused node alone would therefore make
`<input on:focus="…">` fire or not depending on whether the user clicked the
text or the padding beside it — a coin toss dressed as a rule. So the binding
walks.

Walking is safe here in a way it is not for a crossing: exactly one node gains
focus and one loses it, so the walk delivers exactly once per half. A crossing
set is a whole chain, which is why
[`EVENT_POINTER_ENTER`](crate::view::EVENT_POINTER_ENTER) must not walk.

The consequence is stated rather than hidden: `focus` here behaves like the
DOM's `focusin`. That is also why no `focusin` / `focusout` is reserved — it
would be a second name for this one — and the same reasoning keeps
`pointerover` / `pointerout` out.

`target` on the envelope names the node the binding resolved on, which is the
field rather than the text leaf inside it whenever the two differ.
