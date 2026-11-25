`<grid-line>` placement values: the grammar shared by the four
`grid-row-*` / `grid-column-*` longhands and by `grid-auto-flow`
(css-grid-2 §8.3, §7.7).

One edge is `auto`, an integer line index, a named line, or a `span`, and the
same three-token reader produces it whether it arrived through a longhand,
through the `grid-column` / `grid-row` slash form, or as one of the four
edges of `grid-area`. Keeping it here rather than in the shorthand file is
what lets both paths agree by construction.
