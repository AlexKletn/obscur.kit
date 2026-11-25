Post-processes and composites one prepared `filter` sub-scene onto the target
frame: the owned color-matrix pass (if the layer carries one) then the gaussian
blur (if `radius > 0`), sharing the offscreen/composite plumbing.

`s_frame` is the layer's subtree rendered into its margin-padded offscreen (in
absolute coordinates offset so its top-left maps to `origin`). The order is:
color-matrix (into a scratch texture) → blur → composite, matching a
`filter: <color-op> blur()` chain (a color op applied before the spatial
gaussian). A color-only layer (`radius == 0`) composites the color result
directly; a blur-only layer (`color_matrix == None`) is byte-identical to the
shipped blur path.

Returns the color-matrix scratch texture (when one was allocated) so the caller
keeps it alive until the shared encoder is submitted.
