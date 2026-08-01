An opaque identifier for a compositing layer (spec §5.1).

A newtype over a `u32`. Its integer value is an identity token only: callers
mint and compare `LayerId`s but must not attribute meaning to the underlying
number beyond equality and a stable, deterministic ordering. It is `Ord` and
`Hash` so layers can be sorted and keyed in maps.
