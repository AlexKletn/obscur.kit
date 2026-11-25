Maps a `z-index` keyword string to its reserved [`InternedSymbol`].

The only keyword `z-index` accepts is `auto` (mapped to [`Z_INDEX_AUTO`]); the
`<integer>` form lowers to a number, not a keyword. The input is
ASCII-lowercased before matching. Returns [`None`] for any other keyword.
