Maps a `border-*-style` keyword string to its reserved [`InternedSymbol`].

The input is ASCII-lowercased before matching. `none` and `hidden` map to
[`BORDER_STYLE_NONE`]; `dashed` and `dotted` map to [`BORDER_STYLE_DASHED`]
(both painted as a dashed stroke); the remaining continuous styles (`solid` /
`double` / `groove` / `ridge` / `inset` / `outset`) fold to the single
[`BORDER_STYLE_SOLID`] handle, a documented simplification (a solid stroke is
painted). Returns [`None`] for any other keyword.
