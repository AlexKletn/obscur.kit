Builds a scene [`SceneRadialGradient`] fill brush for a style
[`RadialGradient`] over the box `bounds` (spec §5.2, sizing §3.2).

The box-relative center (`center_x` / `center_y`) resolves to an absolute point
against the box; the ending [`shape`](RadialShape) + [`size`](RadialSize)
resolve to a **per-axis** radius pair `(rx, ry)` via [`resolve_radii`]. A
`circle` yields `rx == ry`; an `ellipse` yields independent radii (e.g.
`farthest-corner` on a non-square box gives `rx ≠ ry`), carried faithfully by
the scene [`SceneRadialGradient`] so Vello stretches the concentric circle into
the ellipse. Each style stop's resolved `0.0..=1.0` position becomes the scene
stop's offset; the brush keeps the default
[`ExtendMode::Pad`](obscur_scene::paint::ExtendMode::Pad).
