Creates a layout from its `lines`, `alignment`, and `overflow` policy,
computing the total [`size`](Self::size) from the lines (spec §5.10).

The width is the widest line's width and the height is the sum of the
line heights; an empty paragraph has a zero size.
