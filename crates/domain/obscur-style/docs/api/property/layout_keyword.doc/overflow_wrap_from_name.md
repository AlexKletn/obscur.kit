Maps an `overflow-wrap` (a.k.a. legacy `word-wrap`) keyword string to its
reserved [`InternedSymbol`] (CSS Text 3 §5.2).

The input is ASCII-lowercased before matching. Returns [`None`] for any
keyword outside the closed `normal` / `break-word` / `anywhere` set.
