A control that picks a value — or two — from a scale.

The widget draws nothing itself: it emits a `<canvas>` naming one built-in
painter, carrying the same facts the host node carries. The host's copy is what
the cascade, the accessibility projection and the runtime's drag read; the
canvas's copy is what draws.

Its local name is `slider`, which the accessibility projection maps to
`SemanticRole::Slider` — a role that IS interactive, so Tab reaches it and a
keyboard focus shows a ring, both without this widget arranging anything.
