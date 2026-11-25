The [`TransformObserver`] — watches a node's transform (optional, spec §4.4).

Like all geometry observers it reports the coalesced net change after layout
commit. The watched value is the node's [`AffineTransform`]; the observer
fires whenever the transform changes. `AffineTransform` is `Copy` and
`PartialEq` (six `f32` coefficients), so it is compared and stored directly.
