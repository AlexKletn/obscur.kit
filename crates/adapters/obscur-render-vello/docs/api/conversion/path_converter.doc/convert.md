Converts an obscur-scene [`Path`] into a kurbo [`BezPath`].

Segments are emitted in order: `MoveTo`/`LineTo`/`QuadTo`/`CubicTo`/`Close`
become the corresponding kurbo `move_to`/`line_to`/`quad_to`/`curve_to`/
`close_path` elements. The element count equals the segment count.
