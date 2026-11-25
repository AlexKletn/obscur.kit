Resolves a text node's [`WhiteSpace`] mode from its computed `white-space`
(CSS Text 3 §3).

Reads [`PropertyId::WhiteSpace`] — a [`PropertyValue::Keyword`] backed by one
of the reserved [`layout_keyword`] handles — mapping each `WHITE_SPACE_*`
handle to its variant. An absent property (or any unrecognised keyword)
resolves to [`WhiteSpace::Normal`], the CSS initial, so an unauthored text
node collapses whitespace and soft-wraps.
