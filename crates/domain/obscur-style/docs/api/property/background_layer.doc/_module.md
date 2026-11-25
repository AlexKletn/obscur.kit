One layer of a multi-layer `background-image` list: [`BackgroundLayer`].

CSS `background-image` accepts a **comma-separated list** of image layers,
painted back-to-front (the first-listed layer is the topmost, the last-listed
the bottommost). Each layer is one gradient (`linear-gradient(...)`,
`radial-gradient(...)`, `conic-gradient(...)`, including the `repeating-` forms)
or a `url(...)` reference.

The single-layer case keeps the flat
[`PropertyValue::LinearGradient`](crate::property::PropertyValue::LinearGradient)
/ [`RadialGradient`](crate::property::PropertyValue::RadialGradient) /
[`ConicGradient`](crate::property::PropertyValue::ConicGradient) /
[`BackgroundImageUrl`](crate::property::PropertyValue::BackgroundImageUrl)
variants (byte-identical to before this type existed); a list of two or more
layers is carried as
[`PropertyValue::BackgroundLayers`](crate::property::PropertyValue::BackgroundLayers)
holding an ordered slice of these layers, in author (top→bottom) order.
