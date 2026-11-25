Reads a node's effective CSS `opacity` as a group-layer alpha, or [`None`]
when the subtree needs no opacity layer (spec §5.1).

Reads [`PropertyId::Opacity`] (via [`ComputedStyle::opacity`]) and clamps it to
`0.0..=1.0`. Returns `Some(a)` **only** when `a < 1.0`: a fully-opaque (or
absent) opacity returns [`None`] so nothing is bracketed and the emitted
command stream is byte-for-byte identical to before opacity layers existed. A
partially-transparent value brackets the node's whole subtree with a
`PushLayer(SceneLayer::with_opacity(a))` … `PopLayer`, compositing the group as
one (CSS `opacity` establishes a stacking context and composites the group,
not each descendant independently).
