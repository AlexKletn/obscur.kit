The optional brush transform to pass alongside [`convert`](Self::convert)
when filling/stroking with this brush.

A [`Brush::Radial`] with an **elliptical** ending shape is emitted by
[`convert`](Self::convert) as a concentric *circle* gradient of the x-axis
radius; this transform stretches that circle into the authored ellipse by
scaling the y-axis by `radius_y / radius` about the gradient center (spec
§3.2). Every other brush (solid, linear, conic, or a circular radial) needs
no brush transform and yields [`None`].
