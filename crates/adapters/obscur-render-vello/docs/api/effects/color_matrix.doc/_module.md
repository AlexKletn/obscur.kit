An owned wgpu 4×5 color-matrix pass layered over Vello (spec §5.2).

Vello 0.9 has no color filters, so the CSS color-matrix `filter` functions
(`brightness/contrast/saturate/grayscale/sepia/invert/hue-rotate/opacity`) are
implemented as our **own** wgpu post-process. Because every one of those
functions is an *affine* color transform, the coordinator folds a whole
color-op chain into a single 4×5 matrix (a 4×4 linear part + a 4×1 bias); this
pass applies it to an element's already-rendered sub-scene in **one** fragment
shader (`rgba' = M · rgba + bias`, clamped to `0..1`).

It shares the offscreen / composite plumbing with [`GaussianBlur`](super::GaussianBlur):
the renderer renders a filter layer's subtree into a margin-padded offscreen,
runs this color-matrix pass over it (into a scratch texture), then either blurs
and composites (color + blur) or composites directly (color only). The pipeline
and shader are built once and cached on the renderer; only the small per-call
scratch texture and uniform buffer are per-filter.

# Straight alpha

Vello's texture output is STRAIGHT (non-premultiplied) alpha, which is exactly
the space CSS color filters are defined over, so the matrix is applied directly
to the sampled texels (no un-/re-premultiply).
