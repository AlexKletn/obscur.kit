A CSS `conic-gradient(...)` image, as authored in a stylesheet.

The gradient is described by a starting angle `from_angle_deg` on the CSS
convention — `0` points *up*, increasing **clockwise** — a center position
(`center_x` / `center_y`, box-relative [`Length`]s — the `at <position>`),
and an ordered list of color [`GradientStop`]s. Every parameter is stored
**box-relative** and resolved against the painted box at paint time,
mirroring how [`LinearGradient`] stores only an angle: the scene coordinator
turns the center plus the box's bounds into the concrete center, and the
starting angle into the scene sweep's `start_angle`, that the renderer's
`obscur_scene::paint::ConicGradient` needs. This type is a pure style concept
and deliberately has **no dependency on the scene**.

[`LinearGradient`]: crate::property::LinearGradient

Because [`GradientStop`] carries `f32` data, this type derives [`PartialEq`]
but not [`Eq`]/[`Hash`].
