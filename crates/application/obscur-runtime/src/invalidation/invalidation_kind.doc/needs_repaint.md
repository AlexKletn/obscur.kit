Returns whether a change of this kind requires a repaint pass.

True for [`Paint`](InvalidationKind::Paint) and every deeper kind. A
[`CompositionOnly`](InvalidationKind::CompositionOnly) change needs only a
recompose and therefore returns `false`.
