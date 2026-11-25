Lowers a dimension into a [`Length`] term, accepting `%` (percentage), the
absolute units (`px` / `cm` / `mm` / `Q` / `in` / `pt` / `pc`), the
font-relative units (`em` / `rem` / `ch`), the viewport units (`vw` / `vh` /
`vmin` / `vmax`) and the container units (`cqw` / `cqh` / `cqi` / `cqb` /
`cqmin` / `cqmax`). Any other unit is outside the domain's [`Length`] model
and yields [`None`].

The absolute units fold to `px` immediately, at their fixed css-values-4 §6.2
ratios (see [`absolute_px_per_unit`]). The relative units carry their
coefficient symbolically; they resolve to concrete pixels at the
layout/compute step against a `LengthContext` (spec §4.5).
