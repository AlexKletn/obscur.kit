Arc-length parameterisation of a path, and the slice of it between two fractions.

This is what a progress ring actually is: not a gradient faked into a circle,
but the first `t` of a real path, stroked with real caps. The same measurement
answers dashes and animated outlines, which is why it is a type rather than a
function inside a widget.

**Curves are flattened.** A cubic has no closed-form arc length, so the only
honest way to answer "40% along" is to subdivide until the polyline is within a
tolerance and measure that. The slice comes back as lines for the same reason —
and a stroker consumes lines anyway.

**The whole path is one length domain**, in document order, and a pen jump
between subpaths contributes neither length nor a drawn line. SVG's dash arrays
restart per subpath; this does not, because the useful question here is "how far
along this drawing am I" and a caller who wants per-subpath behaviour can trim
each subpath itself. Stated because the two answers differ and neither is wrong.
