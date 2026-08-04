Whether a node holds a value of its OWN kind rather than text.

`on:input` is how every value-bearing control reports itself, exactly as on the
web — a slider, a checkbox and a field all speak through it. So the binding
alone cannot mean "this is text": a slider carrying one would seat a caret
inside itself and swallow the arrow keys meant to move its thumb. What tells
the two kinds apart is the role, which is the same test the web applies to an
`<input>`'s `type`.
