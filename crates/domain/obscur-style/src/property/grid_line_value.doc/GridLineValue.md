One placement edge of a grid item along a single axis, in style vocabulary.

This is the `obscur-style` twin of `obscur_layout`'s `GridLine`. `obscur-style`
is a pure domain crate that does not depend on `obscur-layout`, so an authored
`grid-column` / `grid-row` edge is modelled here and the runtime converts each
[`GridLineValue`] to the layout crate's own `GridLine` when it bridges a
computed style into a layout input (spec §4.13).

A CSS `grid-column` / `grid-row` is a `start / end` pair; each side is one
[`GridLineValue`]:

* [`Auto`](GridLineValue::Auto) — no explicit line; the item is auto-placed on
  this edge (the initial value).
* [`Line`](GridLineValue::Line) — a **1-based** grid line index (`grid-column: 2`
  is line `2`). A negative index counts back from the end of the explicit grid
  (`-1` is the last line); the layout crate resolves it against the real track
  count. Named lines are out of scope.
* [`Span`](GridLineValue::Span) — `span N`: the edge is placed `N` tracks away
  from the opposite edge.
* [`Named`](GridLineValue::Named) — a `<custom-ident>` line name
  (`grid-column: sidebar-start`). The name cannot be resolved on the item
  alone: it refers to a line named on the item's **container**'s
  `grid-template-columns` / `grid-template-rows` (or an implicit
  `<area>-start` / `<area>-end` line), so the runtime resolves it against the
  parent's computed line-name map, exactly as `grid-area: <name>` is resolved
  against the parent's `grid-template-areas` (spec §4.13).

# Determinism

`GridLineValue` has no interior mutability, so placement resolves
deterministically from the item styles (and, for [`Named`](GridLineValue::Named),
the parent's computed template) alone. The [`Named`](GridLineValue::Named)
variant carries an owned name, so this is [`Clone`] but not [`Copy`].
