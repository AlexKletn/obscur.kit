Creates the node a body `{expr}` interpolation lowers to: a synthetic
element carrying `value` — the interpolated expression (spec §2.8).

A [`Text`](UiIrNodeKind::Text) node holds literal content only, so an
interpolation cannot be one: its content is an expression, and only a
property can carry an expression. Hence a node of its own, paired with
[`interpolated_value`](Self::interpolated_value) — the constructor and the
reader of the same private convention, so the two front ends and the
emitter cannot disagree on the tag by spelling it separately.
