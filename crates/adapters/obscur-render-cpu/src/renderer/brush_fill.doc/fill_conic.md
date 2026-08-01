Rasterizes a conic (angular) gradient over the element outline `sk_path`
(§5.1's "conic gradients"). tiny-skia has no conic shader, so the sweep is
decomposed into angular **wedges** — thin triangles fanning out from the
gradient center — each filled at the color swept at its mid-angle and confined
to the element outline (intersected with the active `clip`).

The gradient center and start angle come from [`ConicGradient`]; a wedge at
normalized offset `o` points along `start_angle + o·2π` (screen-space
clockwise), matching the coordinator's `(from_angle − 90°)` convention and the
`atan2(dy, dx)` a reader would sample with. Wedges cover the full `[0, 1]` turn:
the range before the first stop and after the last is filled with that stop's
color (the [`ExtendMode::Pad`](obscur_scene::paint::ExtendMode) edge), and each
inter-stop span is subdivided finely and its endpoints' colors interpolated, so
a smooth gradient is approximated and a hard-stop gradient (equal endpoint
colors) is exact. Wedges are filled **aliased** so adjacent triangles tile
without a seam; the outline's own anti-aliasing comes from the clip mask.

[`ConicGradient`]: obscur_scene::paint::ConicGradient
