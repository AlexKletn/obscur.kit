Reads a node's computed `mix-blend-mode` as a scene [`BlendMode`], or [`None`]
when the subtree needs no blend layer (compositing-1 §5).

Reads [`PropertyId::MixBlendMode`] — a [`PropertyValue::Keyword`] backed by one
of the reserved [`layout_keyword`] `MIX_BLEND_MODE_*` handles — and maps it to
the matching separable [`BlendMode`]. Returns [`None`] for `normal` (or an
absent value), so nothing is bracketed and the command stream is byte-for-byte
identical to before mix-blend-mode existed. A non-normal value brackets the
node's whole subtree with a `PushLayer(SceneLayer::with_blend_mode(mode))` …
`PopLayer`, compositing the group onto its backdrop with that blend.
