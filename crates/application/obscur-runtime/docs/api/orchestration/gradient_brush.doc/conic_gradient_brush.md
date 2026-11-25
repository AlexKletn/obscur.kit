Builds a scene [`SceneConicGradient`] fill brush for a style
[`ConicGradient`] over the box `bounds` (spec §5.2).

The box-relative center resolves to an absolute point against the box. CSS's
`from` angle is measured from *up* (12 o'clock) increasing clockwise, while the
scene sweep's `0` offset lies along `+x` (3 o'clock) advancing clockwise in
screen space (y down), so the start angle is `from − 90°` in radians. Each
style stop's resolved `0.0..=1.0` position becomes the scene stop's offset; the
brush keeps the default [`ExtendMode::Pad`](obscur_scene::paint::ExtendMode::Pad).
