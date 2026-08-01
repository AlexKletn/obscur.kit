Resolves a text node's [`OverflowWrap`] from its computed `overflow-wrap`
(the legacy `word-wrap` alias lowers to the same property; CSS Text 3 §5.2).

Reads [`PropertyId::OverflowWrap`] — a [`PropertyValue::Keyword`] backed by an
`OVERFLOW_WRAP_*` reserved handle — mapping each handle to its variant. An
absent property (or any unrecognised keyword) resolves to
[`OverflowWrap::Normal`], the CSS initial, so an over-wide word overflows
exactly as before the property existed.
