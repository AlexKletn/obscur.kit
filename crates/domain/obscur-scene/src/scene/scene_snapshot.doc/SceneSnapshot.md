An **immutable, versioned** snapshot of a [`Scene`], the hand-off a renderer
backend consumes (spec §1.6, §5.2).

A snapshot is **immutable after publication**: it exposes only read-only
accessors and no mutators. The runtime may retain it and later discard it as
stale once the live scene's generation advances past it — see
[`SceneSnapshot::is_stale_against`] (spec §1.6).

The snapshot is the exact contract a renderer honours: its
[`DisplayList`] fixes the geometry, paint order, and clipping that every
backend must reproduce. Rasterization may differ between backends, but the
scene geometry and command order must not (spec §5.2).
