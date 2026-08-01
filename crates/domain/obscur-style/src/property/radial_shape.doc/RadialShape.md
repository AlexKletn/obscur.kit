The ending shape of a CSS `radial-gradient(...)`: a `circle` or an `ellipse`.

The gradient's ending shape is either a circle (equal horizontal/vertical
radius) or an ellipse (axis-aligned, independent radii). The CSS initial —
used when neither keyword is authored — is [`RadialShape::Ellipse`].

The scene coordinator resolves the shape to a per-axis radius pair `(rx, ry)`:
a `circle` yields `rx == ry`, an `ellipse` independent radii (spec §3.2). The
scene radial gradient carries both, so an ellipse renders faithfully (stretched
vertically) on Vello.
