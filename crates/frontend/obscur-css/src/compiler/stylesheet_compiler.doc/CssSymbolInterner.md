A string interner mapping CSS text to opaque [`InternedSymbol`] handles.

The kit exposes no shared global interner, so the CSS compiler owns this one.
Symbols are deduplicated, so equal strings always intern to the same symbol.
The reverse table lets consumers [`resolve`](CssSymbolInterner::resolve) a
symbol back to its text.

A symbol is minted with [`InternedSymbol::from_index`], **not** `from_raw`:
this table numbers from zero, and a raw zero-based id lands inside the low
range the reserved symbols occupy, where a long enough sheet reached
`layout_keyword::OVERFLOW_HIDDEN` and became indistinguishable from it.
`from_index` lifts the id into the same high namespace
[`InternedSymbol::named`] uses, which keeps the two ranges disjoint by
construction rather than by margin.
