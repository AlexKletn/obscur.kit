Maps a `text-align` keyword string to its reserved [`InternedSymbol`].

The input is ASCII-lowercased before matching. Returns [`None`] for any
keyword outside the closed `left` / `right` / `center` / `justify` / `start` /
`end` set.
