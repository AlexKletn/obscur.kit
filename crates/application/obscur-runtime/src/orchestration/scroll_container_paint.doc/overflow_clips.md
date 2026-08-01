Whether a node's computed `overflow` clips its subtree to its box.

Reads [`PropertyId::Overflow`] — a [`PropertyValue::Keyword`] backed by one of
the reserved [`layout_keyword`] `OVERFLOW_*` handles. Every value except
`visible` clips (`hidden` / `clip` / `scroll` / `auto`; the last two clip like
`hidden`, as no scrollbars are synthesised yet). An absent property is the CSS
initial `visible` — no clip, byte-for-byte identical to before overflow existed.
