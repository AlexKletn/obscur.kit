Copies a rendered frame onto the surface, multiplying colour by alpha on the way.

**Why this exists instead of `wgpu::util::TextureBlitter`.** Vello writes
STRAIGHT alpha. A Wayland swapchain offers `Opaque` and `PreMultiplied` and
nothing else, and under `PreMultiplied` the compositor computes
`dst = src.rgb + (1 - src.a) * dst` without scaling `src.rgb` first. Handing it
straight alpha therefore paints a translucent surface as an opaque one,
brightened by whatever was behind it: a card authored at `alpha: 0.72` over a
dark desktop measured `(255, 252, 249)` where an honest blend is `(187, 183, 181)`.
The stock blitter copies verbatim, so the multiply has to happen in a pass of
our own.

The same defect, with the same measurement, was found and fixed once before in
this toolkit's first version. It is a property of the boundary — Vello's output
convention against Wayland's surface convention — not of either side's bug, so
whatever presents has to carry it.
