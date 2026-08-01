A vector outline (spec §5.1): an ordered sequence of [`PathSegment`]s plus a
[`FillRule`] describing how the interior is filled.

A `Path` is plain, backend-independent data. It carries no rendering state
and references no GPU or renderer types; a backend interprets the segments.
