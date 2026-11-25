Resolves a list of tracks to concrete pixel sizes along one axis via the
canonical CSS-grid track-sizing algorithm (§11).

`available` is the content extent along the axis and `gap` the inter-track
gap. The `collapsed` mask has one entry per track. Collapsed tracks have
zero size. The other tracks share the available space after subtracting one
gap between each pair of consecutive active tracks.
`definite` is whether the container's size along this axis is authored (not
`auto`) and `stretch` whether its content alignment routes to §11.8 (the
`stretch`/`normal` default). `items` are the grid items as
`(start_track, span, contributions)`; only items crossing an intrinsic track
need a real contribution for the [`Kind`]s they cross. On an unbounded axis,
items crossing flexible tracks also supply a max-content contribution.
`content_bounds` constrains the intrinsic flex result and includes gap space.

Simple cases — fixed tracks, a single `1fr` split with no floors, and
non-spanning items — resolve to the same sizes as the previous single-pass
model; fr-restart, span distribution, maximise, and stretch change only the
cases they were added to fix.
