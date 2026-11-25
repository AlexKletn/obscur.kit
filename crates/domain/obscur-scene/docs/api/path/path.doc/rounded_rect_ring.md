Builds a closed rounded-rectangle **ring**: an `outer` rounded rectangle
with an `inner` rounded rectangle punched out as a hole, filled with the
[`FillRule::EvenOdd`] rule so the inner rectangle's interior is excluded.

The result is a single path holding two subpaths — the outer rounded rect
followed by the inner rounded rect (each built by [`Path::rounded_rect`]).
Under the even-odd rule the region between the two outlines is filled while
the inner region is left empty, yielding a rounded border ring whose OUTER
**and** INNER corners are rounded. Callers pass the border-box rect with the
authored corner radii as `outer`, and the inner (padding-box) rect with
per-corner radii reduced by the border width as `inner`.
