Derives a [`LayoutStyle`] for every node in `computed`.

Each node's [`ComputedStyle`](obscur_style::computed::ComputedStyle) is mapped independently via
[`layout_style_from`]; nodes absent from `computed` simply produce no
entry in the result.

`viewport` and `root_font_size` supply the resolution context for
relative and viewport-relative units and `calc()` expressions (spec
§4.5): `vw`/`vh`/`vmin`/`vmax` resolve against `viewport`, `rem` against
`root_font_size`, and `em` against each node's own computed `font-size`.
Compute `root_font_size` with [`root_font_size`](super::root_font_size) from the tree root.

`paragraph` resolves `ch`, the one unit that cannot be folded from numbers
alone — it needs the `0` glyph shaped (see [`ch_width_for`](super::length_resolution::ch_width_for), which measures
only for the rare node that writes `ch`). Pass [`None`] when no shaper is
available; `ch` then takes the specified `0.5em` substitute and every other
unit is unaffected.
