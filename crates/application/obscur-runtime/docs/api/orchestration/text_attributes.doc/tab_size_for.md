Resolves a text node's [`TabSize`] from its computed `tab-size` (CSS Text 3
§4.3).

Reads [`PropertyId::TabSize`]: a unitless [`PropertyValue::Number`] resolves to
[`TabSize::Factor`] (a multiple of the space advance), a [`PropertyValue::Length`]
to [`TabSize::Length`] with relative units resolved against the node's font
context — `px` verbatim, `em` × `font_size`, `rem` × `root_font_size` (CSS
Values 4 §5.1.1). An absent property, the initial, a percentage (invalid for
`tab-size` per CSS Text 3 §4.3 — `<number>` or `<length>` only), or a viewport
unit resolves to the initial `8` spaces
([`TabSize::Factor(8.0)`](TabSize::Factor)). Tabs only take effect under a
preserving `white-space` mode (`pre` / `pre-wrap`); under a collapsing mode a
tab has already become a space, so this value is unused.
