A gradient whose color varies radially outward from `center` (spec §5.1).

Offset `0.0` maps to `center` and offset `1.0` maps to the ending shape of
horizontal radius `radius` and vertical radius `radius_y`; the color at any
point depends on its normalized distance from `center`. Behavior beyond the
ending shape (positions past offset `1.0`) is governed by the [`ExtendMode`].

The ending shape is a **circle** when `radius_y == radius` and an axis-aligned
**ellipse** otherwise (`radius` is the x-axis radius, `radius_y` the y-axis
radius) — see [CSS Images §3.2]. Color-stop offsets are measured along the
x-axis radius; a backend renders the ellipse by stretching the concentric
circle of radius `radius` vertically by `radius_y / radius`.

This is a simple **concentric** radial gradient: the color rings are centered
on a single point. A distinct focal point (an offset origin for the `0.0`
ring) is a later extension.

Stops are stored in ascending `offset` order (callers supply them sorted;
see [`GradientStop`]).

[CSS Images §3.2]: https://drafts.csswg.org/css-images-3/#radial-gradients
