§4.7: "Hit testing accounts for:" / "- transforms;" and "An incremental
spatial index is used."

A 100x40 box rotated 45° about its centre (50, 20). (50, 20) hits it.
(2, -28) lies inside the rotated AABB (`x ∈ [0.5, 99.5]`,
`y ∈ [-29.5, 69.5]`, half-extents `(100·cos45° + 40·sin45°)/2 = 49.5`) but
inverse-rotating it about the centre gives a local x of `50 − 48·√2 ≈ −17.9`
— outside `[0, 100]` — so it must miss.

Three probes, because the first one alone is not a proof. (2, -28) is
outside the *untransformed* rect as well, so a hit test that never looks
at the transform passes it by accident. The two that discriminate:

* **(2, 2)** — inside the layout rect, so the transform-blind answer is a
  hit; inverse-rotated it is local (3.3, 41.2), past the box's 40px bottom
  edge, so the right answer is a miss. The vacated corner of a rotated
  card must not be clickable.
* **(92.4, 41.2)** — *outside* the layout rect (`y > 40`), so the
  transform-blind answer is a miss; it is where local (95, 5) — real,
  painted body of the card — was carried to, so the right answer is a hit.
  This is also what forbids "fix" the index by shrinking: the broad phase
  has to grow to the transformed corners, not shrink to the shape.
