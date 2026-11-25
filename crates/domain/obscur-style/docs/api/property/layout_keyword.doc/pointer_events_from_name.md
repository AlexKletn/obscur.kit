Maps a `pointer-events` keyword string to its reserved [`InternedSymbol`].

The input is ASCII-lowercased before matching. Returns [`None`] for any
keyword outside the closed `auto` / `none` set — including the SVG-only
values (`visiblePainted`, `visibleFill`, `visibleStroke`, `visible`,
`painted`, `fill`, `stroke`, `all`), which name fill/stroke/`visibility`
distinctions this renderer has no notion of.
