A string interner mapping CSS text to opaque [`InternedSymbol`] handles.

The kit exposes no shared global interner, so the CSS compiler owns this one.
Symbols are minted with sequential raw ids and deduplicated, so equal strings
always intern to the same symbol. The reverse table lets consumers
[`resolve`](CssSymbolInterner::resolve) a symbol back to its text.
