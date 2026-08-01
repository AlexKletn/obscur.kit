Maps a `display` keyword string to its reserved [`InternedSymbol`].

The input is ASCII-lowercased before matching, so callers may pass authored
casing directly. Returns [`None`] for any keyword outside the closed set
modelled here.
