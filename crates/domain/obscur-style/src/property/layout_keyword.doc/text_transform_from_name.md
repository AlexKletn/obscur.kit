Maps a `text-transform` keyword string to its reserved [`InternedSymbol`]
(CSS Text 3 §2.1).

The input is ASCII-lowercased before matching. Returns [`None`] for any
keyword outside the closed `none` / `uppercase` / `lowercase` / `capitalize`
set (`full-width` / `full-size-kana` are not modelled).
