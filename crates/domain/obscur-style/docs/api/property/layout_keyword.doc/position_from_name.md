Maps a `position` keyword string to its reserved [`InternedSymbol`].

The input is ASCII-lowercased before matching. Returns [`None`] for any
keyword outside the closed `static` / `relative` / `absolute` / `fixed` /
`sticky` set.
