The inline space offered to content that is being asked how *wide it wants to
be* — unbounded, so nothing wraps that does not have to.

Infinity rather than a large finite number on purpose. A sentinel like
`1e6` is a number the layout can accidentally *use*: a `50%` child would
resolve against it and come out 500000px, and a `max-content` box would
silently be half a million pixels wide instead of visibly wrong. Infinity
cannot be mistaken for a real length —
[`Dimension::resolve`](crate::model::Dimension::resolve) tests `is_finite` and
treats a percentage against it as `auto`, which is exactly what CSS Sizing 3
§5.1 requires of an indefinite basis.
