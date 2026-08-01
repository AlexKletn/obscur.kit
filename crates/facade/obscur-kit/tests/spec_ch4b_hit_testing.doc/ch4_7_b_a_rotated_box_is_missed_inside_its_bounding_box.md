§4.7: "Hit testing accounts for:" / "- transforms;" and "An incremental
spatial index is used."

A 100x40 box rotated 45° about its centre (50, 20). (50, 20) hits it.
(2, -28) lies inside the rotated AABB (`x ∈ [0.5, 99.5]`,
`y ∈ [-29.5, 69.5]`, half-extents `(100·cos45° + 40·sin45°)/2 = 49.5`) but
inverse-rotating it about the centre gives a local x of `50 − 48·√2 ≈ −17.9`
— outside `[0, 100]` — so it must miss.
