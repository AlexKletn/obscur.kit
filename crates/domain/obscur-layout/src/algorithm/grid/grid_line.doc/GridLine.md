One placement edge of a grid item along a single axis (spec §4.13, a subset).

A CSS `grid-column` / `grid-row` is a `start / end` pair; each side is one
`GridLine`. This models the three forms this crate supports:

* [`Auto`](GridLine::Auto) — no explicit line; the item is auto-placed on this
  edge (the initial value).
* [`Line`](GridLine::Line) — a **1-based** grid line index (`grid-column: 2`
  is line `2`, i.e. the start of the second track). A **negative** index counts
  back from the end of the explicit grid (`-1` is the last line); it is resolved
  against the real track count during placement. A zero index is clamped to line
  `1`. Named lines are out of scope.
* [`Span`](GridLine::Span) — `span N`: the edge is placed `N` tracks away from
  the opposite edge. A zero span is clamped to `1`.

# Determinism

`GridLine` is a plain `Copy` value with no interior mutability, so placement
resolves deterministically from the item styles alone.
