An owned wgpu separable-gaussian blur pass layered over Vello (spec §5.2).

Vello 0.9 has no content-blur primitive, so `filter: blur()` is implemented as
our **own** wgpu post-process: a blur layer's subtree is rendered to an
offscreen texture, run through a horizontal then a vertical gaussian pass (a
WGSL fragment shader with gaussian weights derived from the radius, ping-ponging
between textures), and composited back onto the target with premultiplied-over
alpha blending. The pipelines and shader are built once and cached on the
renderer; only the small per-blur textures and uniform buffers are per-call.

# Scope and limitations (documented, honest)

- **Z-order.** The blurred result is composited **on top** of the target at its
  bounds. This is correct for the common case — a blurred panel/card with
  nothing painted above it — but a later sibling that should paint over the
  blurred element would be overdrawn. Full stacking-correct blur needs the
  effect folded into the compositor, a follow-up.
- **Margin / edges.** The offscreen texture is padded by `ceil(3 * radius)` on
  every side so the gaussian tail is not clipped at the element's edge; the
  sampler clamps to a transparent border, so nothing bleeds in from outside.
- **Coordinate space.** Bounds are the element's *untransformed* border box and
  the target is assumed 1:1 logical↔physical pixels; a device scale or an
  enclosing transform combined with blur is approximate (documented).
- **`backdrop-filter`** (blurring the content BEHIND the element) reuses this
  same pipeline set via [`GaussianBlur::render_backdrop`]: the renderer's
  segmented compositing path copies a region of the accumulated frame, runs the
  same separable gaussian over it, and composites it back masked to the panel's
  rounded shape (the frosted-glass / acrylic effect, spec §5.2).
