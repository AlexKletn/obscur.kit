How a container aligns its items along the cross axis (flex, spec §4.1) or a
grid item within its cell on one axis (the container default, spec §4.1 grid).

The `FlexStart` / `FlexEnd` spellings are the flexbox cross-axis values; the
`Start` / `End` spellings are the grid (and CSS-align) values. They are kept
as distinct variants so flex layout can key off its own pair without a grid
value ever perturbing it; a grid formatting context treats `FlexStart` and
`Start` (and `FlexEnd` / `End`) as the same physical alignment.
