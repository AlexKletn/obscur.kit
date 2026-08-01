Extracts `property`'s computed value from `style` as the [`MotionValue`] that
`kind` animates, or [`None`] when the property is absent or not that value
shape.

An [`AnimatedKind::Number`] reads a bare [`PropertyValue::Number`]; an
[`AnimatedKind::LengthPx`] reads an absolute pixel length (a percentage or
`auto` length has no fixed scalar and yields [`None`]); an
[`AnimatedKind::Transform`] reads the resolved 2D affine, so a change is
detected on the whole matrix and the transition interpolates through it by
decomposition (CSS Transforms §14).
