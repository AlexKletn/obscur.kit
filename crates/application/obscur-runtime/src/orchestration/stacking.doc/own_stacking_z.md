Reads a node's **own** stacking z: its numeric `z-index`, but only when the
node is positioned (`position != static`).

Returns [`None`] for a static node, a positioned node whose `z-index` is
`auto` (or absent), or any node without a computed style — in every case the
node inherits its parent's effective z rather than establishing its own level.
