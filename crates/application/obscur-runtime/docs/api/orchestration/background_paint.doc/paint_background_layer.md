Paints one `background-image` [`BackgroundLayer`] over a node's border box
`bounds`: a gradient fills the box (rounded when `has_radius`), a `url(...)`
layer draws its resolved image with the node's `background-size`/`-position`/
`-repeat` geometry (spec §5.2).

A stop-less gradient (never produced by the CSS lowering) and a `url(...)`
whose image is absent from `images` (unloaded / failed) both paint nothing,
letting the layers below show through.
