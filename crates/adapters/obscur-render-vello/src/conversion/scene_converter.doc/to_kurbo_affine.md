Converts an obscur-geometry [`AffineTransform`](obscur_geometry::AffineTransform)
into a kurbo [`Affine`]. Both store the six coefficients `[a, b, c, d, e, f]`
in the identical layout, so the conversion is a straight widening to `f64`.
