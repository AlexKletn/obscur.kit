Interpolates two 2D decompositions at progress `t` (CSS Transforms §14.4).

`translate`, `scale` and `skew` lerp component-wise. The rotation `angle`
takes the **shortest arc**: if the endpoints differ by more than `π` the
smaller is shifted by `+2π` so the blend rotates the short way (never the
long way around), matching the spec's "the rotation is interpolated along
the shortest path" requirement — e.g. `10°→350°` sweeps `−20°` through `0°`
rather than `+340°`.

`t` follows the crate's [`Interpolatable`](super::Interpolatable) contract:
normalized progress, **not** clamped, so easing/spring overshoot extrapolates.
