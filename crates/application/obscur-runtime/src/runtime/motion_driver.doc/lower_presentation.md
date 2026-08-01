Lowers a sampled [`MotionValue`] into the [`PropertyValue`] a
[presentation property](is_presentation_property) presents, keyed off the
**property** rather than any [`AnimatedKind`]: `transform` presents the
interpolated affine as a [`PropertyValue::Transform`], every other
presentation property (i.e. `opacity`) presents the scalar as a
[`PropertyValue::Number`]. Returns [`None`] when the sample's variant does
not match (e.g. a scalar sample for `transform`).
