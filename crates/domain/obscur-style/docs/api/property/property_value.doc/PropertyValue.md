A typed CSS property value.

Because [`PropertyValue::Number`], [`PropertyValue::Length`] and
[`PropertyValue::Color`] carry `f32` data, this type derives [`PartialEq`]
but not [`Eq`]/[`Hash`].
