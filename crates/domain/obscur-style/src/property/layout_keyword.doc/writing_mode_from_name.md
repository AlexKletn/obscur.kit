Maps a `writing-mode` keyword string to its reserved [`InternedSymbol`].

The input is ASCII-lowercased before matching. Returns [`None`] for any
keyword outside the closed `horizontal-tb` / `vertical-rl` / `vertical-lr` set.
