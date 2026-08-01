Applies this delta to `props`, writing only the fields that are set.

Unchanged (`None`) fields are left as-is. For the position, an outer
`Some(inner)` overwrites [`WindowProperties::position`] with `inner`
(an explicit point or a platform-chosen `None`).
