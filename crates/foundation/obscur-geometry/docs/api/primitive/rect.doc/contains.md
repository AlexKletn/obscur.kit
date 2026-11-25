Returns whether the (fractional) point lies within the rectangle.

Uses exact fractional geometry — hit testing must not pre-round
(spec §4.6).

The rectangle is half-open on both axes: `[origin.x, right())` by
`[origin.y, bottom())`. The leading edges belong to the rectangle and the
trailing edges do not, so a point on the seam between two abutting
rectangles is claimed by exactly one of them — the later one. A rectangle
with zero width or zero height therefore contains no point at all.
