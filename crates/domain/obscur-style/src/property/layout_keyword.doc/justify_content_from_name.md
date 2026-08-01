Maps a `justify-content` keyword string to its reserved [`InternedSymbol`].

The input is ASCII-lowercased before matching. Both the modern (`start` /
`end`) and the flexbox (`flex-start` / `flex-end`) spellings resolve to the
same reserved handle. Returns [`None`] for any keyword outside the closed set
modelled here.
