Linearly interpolates between two `f32` endpoints.

Returns `a + (b - a) * t`, the scalar lerp used by every
[`Interpolatable`] impl in this module. `t` is the normalized progress in
`[0, 1]` (see [`Interpolatable::interpolate`]) but is deliberately **not**
clamped: easing curves, springs and overshoot legitimately pass `t` outside
`[0, 1]`, and this helper extrapolates linearly past the endpoints.
