The gap between a layer surface and each edge it is anchored to.

A margin against an edge the surface is not anchored to has no effect — there
is nothing for it to measure from. Signed, because a negative margin pushing a
surface partly off-screen is a real thing a dock does when it hides.

Named `LayerMargins` rather than `Margins` because
[`obscur-layout`](https://docs.rs/obscur-layout) already has a `Margins` that
means the CSS box-model margin, and the two are not the same idea.
