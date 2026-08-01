Maps a `white-space` keyword string to its reserved [`InternedSymbol`]
(CSS Text 3 §3).

The input is ASCII-lowercased before matching. Returns [`None`] for any
keyword outside the closed `normal` / `nowrap` / `pre` / `pre-wrap` /
`pre-line` set (`break-spaces` is not modelled).
