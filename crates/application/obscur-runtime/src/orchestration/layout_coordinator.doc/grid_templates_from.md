Derives a [`GridTemplates`] side-table for every grid node in `computed`
that authored a `grid-template-columns` / `grid-template-rows`.

This is the grid twin of
[`layout_styles_from`](Self::layout_styles_from): the same `viewport` /
`root_font_size` resolution context folds a fixed track's relative units to
pixels. Nodes with no authored template produce no entry (and fall back to
the default single implicit column at dispatch).
