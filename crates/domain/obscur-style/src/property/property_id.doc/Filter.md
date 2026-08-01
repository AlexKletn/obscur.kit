The `filter` property (a `BoxShadow` list value carrying only the
`drop-shadow(...)` functions; NOT inherited). Only the single
GPU-expressible filter function, `drop-shadow(<ox> <oy> <blur>? <color>?)`,
is modelled — it lowers to a drop [`BoxShadow`](crate::property::BoxShadow)
(spread `0`, not inset) that the scene coordinator paints BEHIND the
element like a `box-shadow` drop shadow. Non-drop-shadow functions
(`blur()`, `brightness()`, …) are out of scope on vello 0.9 and are
silently dropped; `filter: none` / empty lowers to an empty list.
