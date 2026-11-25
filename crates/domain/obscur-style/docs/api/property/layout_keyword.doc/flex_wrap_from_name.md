Maps a `flex-wrap` keyword string to its reserved [`InternedSymbol`].

The input is ASCII-lowercased before matching. Returns [`None`] for any
keyword outside the closed `nowrap` / `wrap` / `wrap-reverse` set.
