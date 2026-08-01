The `(leading, extra_gap)` a content-distribution mode contributes when the
tracks along one axis leave leftover free space (spec §10.4, `align-content` /
`justify-content` on a grid container).

`n` non-collapsed tracks occupy `Σ sizes + gap * (n − 1)` of the axis; the
`leftover = content_extent − that` (when positive) is distributed:
* `FlexStart` — nothing (the historical packing; both `0.0`).
* `FlexEnd` — a `leading` of the full leftover, no extra gap.
* `Center` — a `leading` of half the leftover, no extra gap.
* `SpaceBetween` — no leading, an extra `leftover/(n−1)` between tracks (n≥2;
  n<2 falls back to `FlexStart`).
* `SpaceAround` — a `leftover/(2n)` half-gap at each end plus `leftover/n`
  between adjacent tracks.
* `SpaceEvenly` — an equal `leftover/(n+1)` gap at the ends and between tracks.

A non-positive leftover, an empty axis, or the `FlexStart` default all yield
`(0.0, 0.0)`, so a default grid is byte-identical to the un-distributed
packing.
