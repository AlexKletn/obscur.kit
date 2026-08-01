Clamps a main-axis size into a pre-resolved `[min_floor, max_val]` range
(spec §4.6).

`min_floor` and `max_val` are the item's main-axis bounds already resolved to
pixels against the container main extent (with `max_val` set to
[`f32::INFINITY`] when the item has no `max`). It applies `max` first, then the
`min` floor — so `min` wins on conflict, matching CSS `min-width` overriding
`max-width` — and never returns a negative size. `min_floor` carries the §4.5
content-based automatic minimum on the `auto` path and the explicit `min`
otherwise, so an `auto` min with no content floor (`min_floor == 0`) and no
`max` leaves the value unchanged.
