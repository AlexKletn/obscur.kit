Frosts the **backdrop** in place: blurs the region of the accumulating frame
`accum` under `bounds` (the panel's border box, physical pixels
`(x0, y0, x1, y1)`), clipped to the rounded `radii`, and writes the blurred
result back over `accum` (spec §5.2, CSS `backdrop-filter`).

Unlike [`Self::render`] (which blurs a *sub-scene* and composites it), this
samples the already-composited frame itself: a padded region of `accum` is
copied into a scratch texture (the `~3σ` margin keeps the gaussian tail from
clipping at the panel edge, clamped to the frame), run through the same
separable gaussian, and composited back over `accum` masked to the rounded
panel shape — so only the panel area is frosted and its corners stay sharp
outside the radius. Recorded into `encoder`; the caller submits after the
prior segment composites so the backdrop is fully painted first.
