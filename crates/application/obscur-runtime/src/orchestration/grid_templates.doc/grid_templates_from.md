Builds a [`GridTemplates`] side-table from every node's [`ComputedStyle`].

Only nodes that authored `grid-template-columns` and/or
`grid-template-rows` produce an entry; a node with neither is skipped, so the
map stays small. `viewport`, `root_font_size` and `paragraph` supply the
resolution context for relative / viewport units in a fixed track (`em`,
`rem`, `vw`, `ch`, …), exactly as in
[`layout_styles_from`](super::LayoutCoordinator::layout_styles_from) — so
`grid-template-columns: repeat(3, 20ch)` sizes its tracks from the `0` glyph.
