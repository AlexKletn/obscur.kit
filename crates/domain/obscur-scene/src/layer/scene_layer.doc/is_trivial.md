Returns whether the layer is trivial: identity transform, no clip,
opacity `>= 1.0`, [`BlendMode::Normal`], [`CompositionMode::SrcOver`],
and no effects.

A trivial layer changes nothing about how its content composites, so the
renderer can flatten it away without an offscreen pass.
