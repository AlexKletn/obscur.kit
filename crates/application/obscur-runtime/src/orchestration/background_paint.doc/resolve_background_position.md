Resolves one `background-position` component to the tile's origin offset from
the box edge, following the CSS position formula: an absolute `px` is that
offset directly; a `Percent` aligns the image's `pct` point to the box's `pct`
point, i.e. `(box_extent − tile_extent) × pct`. Any other (context-dependent)
length falls back to `0` (the top/left edge, a documented approximation).
