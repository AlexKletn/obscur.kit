Lowers a CSS `transform-origin` value into a [`PropertyValue::TransformOrigin`]
(spec §5.2).

A one-value form sets the axis the component names (a horizontal keyword or a
length/percentage → `x`, a vertical keyword → `y`), the other axis defaulting to
`50%` (center). A two-value form is `<x> <y>`, but the `top left`-style keyword
order is also accepted (a vertical first component and horizontal second are
swapped). Lengths are `px`/`%` (a unitless `0` is `0px`); the `left`/`center`/
`right` and `top`/`center`/`bottom` keywords lower to `0%`/`50%`/`100%`. Returns
[`None`] for an unparseable or over-long value.
