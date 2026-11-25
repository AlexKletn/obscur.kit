Maps a `mix-blend-mode` keyword string to its reserved [`InternedSymbol`].

The input is ASCII-lowercased before matching. Returns [`None`] for any keyword
outside the separable set modelled here (`normal` plus the separable modes).
