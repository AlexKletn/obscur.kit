Resolves a text node's [`TextOverflow`] policy from its computed
`text-overflow` (§4.14, §5.10).

Reads [`PropertyId::TextOverflow`] — a [`PropertyValue::Keyword`] backed by one
of the reserved [`layout_keyword`] handles — mapping `TEXT_OVERFLOW_ELLIPSIS`
to [`TextOverflow::Ellipsis`] and everything else (including `TEXT_OVERFLOW_CLIP`
and an absent property) to [`TextOverflow::Clip`], the CSS initial. The same
value drives both the layout measure/arrange pass and the scene compose pass so
the sized box and the painted (possibly truncated) lines agree.
