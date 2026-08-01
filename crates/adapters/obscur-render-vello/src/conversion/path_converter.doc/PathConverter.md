A stateless converter from an obscur-scene [`Path`] to a kurbo [`BezPath`].

obscur-scene coordinates are `f32` in logical-pixel space; kurbo works in
`f64`, so each coordinate is widened. Each [`PathSegment`] maps to exactly one
kurbo path element, so segment order and count are preserved.
