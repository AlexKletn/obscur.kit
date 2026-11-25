Resolves a text node's [`TextTransform`] from its computed `text-transform`
(CSS Text 3 §2.1).

Reads [`PropertyId::TextTransform`] — a [`PropertyValue::Keyword`] backed by
one of the reserved [`layout_keyword`] handles — mapping each
`TEXT_TRANSFORM_*` handle to its variant. An absent property (or any
unrecognised keyword) resolves to [`TextTransform::None`], so an unauthored
text node's string is shaped unchanged.
