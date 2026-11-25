Color space in which gradient stops are interpolated.

Per **CSS Images Module Level 4 §3.4** ("Color Stops and Transition Hints"),
a gradient with no authored `in <color-space>` interpolates its stops in
**Oklab** — the default rectangular color-interpolation space for `color-mix`
and gradients (CSS Color 4 §12). peniko itself defaults a `Gradient` to sRGB
(`ColorSpaceTag::Srgb`), which produces the muddy dark midpoint of e.g.
`linear-gradient(red, green)`.

We set this tag on every gradient (forward-compatible with a Vello that honors
it), but this Vello/peniko build **rasterizes the ramp in sRGB regardless of
the tag** (verified by the `gradient_oklab_proof` example: the tag alone left
the red→green midpoint at the muddy sRGB `(127, 64, 0)`). So the CSS-correct
interpolation is produced by **densifying** the stops in Oklab (see
[`BrushConverter::stops`]) before handing them to peniko; the tag is kept as a
belt-and-suspenders default that stays correct if Vello later honors it.

Hue direction stays peniko's default (`shorter`); Oklab is rectangular so hue
direction is irrelevant here. Parsing an explicit `in <color-space>` from the
stylesheet is a future extension — the domain model does not yet carry an
authored interpolation space, so Oklab (the CSS default) is applied
unconditionally.
