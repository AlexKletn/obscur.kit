Resolves the nearest editable ancestor of `target` (target first, toward the
root): the first node whose own bindings carry
[`EVENT_INPUT`](obscur_view::view::EVENT_INPUT) **and** which is not a control
that holds its own value. Returns `None` when no node in the chain qualifies.

The second condition is not a redundancy. `on:input` is how *every*
value-bearing control reports itself, exactly as on the web — a slider, a
checkbox and a text field all speak through it — so the binding alone cannot
mean "this is text". What tells the two kinds apart is the semantic role, which
is the same test the web applies to an `<input>`'s `type`: a node whose role is
`Slider`, `ProgressBar`, `Checkbox`, `RadioButton`, `Button` or `Link` is
skipped, however it is bound.

The defect that closed: a `Slider` carrying `on:input` resolved as editable, so
a press seated a caret inside it and the editing session swallowed the arrow
keys meant to move its thumb.
