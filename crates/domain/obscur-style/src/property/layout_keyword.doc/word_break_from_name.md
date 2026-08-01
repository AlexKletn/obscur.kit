Maps a `word-break` keyword string to its reserved [`InternedSymbol`]
(CSS Text 3 §5.2).

The input is ASCII-lowercased before matching. Returns [`None`] for any
keyword outside the closed `normal` / `break-all` / `keep-all` set
(`break-word`, a deprecated `word-break` value, is not modelled here).
