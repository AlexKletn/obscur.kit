A grid item's explicit line-based placement (spec §4.13, a subset).

Each axis is a `start / end` [`GridLine`] pair — the CSS `grid-column`
(`column_start / column_end`) and `grid-row` (`row_start / row_end`). The
[`Default`] is all [`GridLine::Auto`], i.e. pure row-major auto-placement (the
grid's historical behaviour), so a default `GridPlacement` never perturbs an
existing layout.

# Resolution

[`resolve_axis`](GridPlacement::resolve_column) turns one axis pair into a
concrete `(start, span)`:

* `Line(a) / Line(b)` → definite start `a - 1`, span `max(b - a, 1)`.
* `Line(a) / Span(n)` → definite start `a - 1`, span `n`.
* `Line(a) / Auto`    → definite start `a - 1`, span `1`.
* `Span(n) / Line(b)` → definite start `(b - 1) - n`, span `n` (end-anchored).
* `Auto / Line(b)`    → definite start `b - 2`, span `1` (end-anchored).
* `Auto / Span(n)`    → auto-placed (start `None`), span `n`.
* `Auto / Auto`       → auto-placed (start `None`), span `1`.

A `None` start means the axis is auto-flowed; a `Some(i)` start is a definite
0-based track index. Spans are always `≥ 1`.

# Determinism

`GridPlacement` is a `Copy` value with no interior mutability; placement
resolves deterministically from the item styles alone.
