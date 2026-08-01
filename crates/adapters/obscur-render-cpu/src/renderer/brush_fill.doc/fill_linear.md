Rasterizes a linear (axial) gradient over the element outline `sk_path` for real,
using tiny-skia's linear shader (§5.1's "linear gradients").

The authored stops are expanded into a dense stop list sampled along the
**premultiplied** interpolation curve ([`premul_lerp_color`]), so tiny-skia's
straight-alpha interpolation between the dense samples reproduces the
premultiplied result css-color-4 §12.3 requires. The gradient axis (`start` →
`end`) and stop offsets come straight from the [`LinearGradient`]; `transform`
maps both the path and the shader, so the gradient tracks any active CSS 2D
transform. A gradient with fewer than two usable stops falls back to a flat fill
of its first stop (`brush_color`), matching the stop-less/solid degenerate cases.

[`LinearGradient`]: obscur_scene::paint::LinearGradient
