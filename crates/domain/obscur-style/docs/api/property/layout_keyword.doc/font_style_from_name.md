Maps a `font-style` keyword string to its reserved [`InternedSymbol`].

The input is ASCII-lowercased before matching. Returns [`None`] for any
keyword outside the closed `normal` / `italic` / `oblique` set.
