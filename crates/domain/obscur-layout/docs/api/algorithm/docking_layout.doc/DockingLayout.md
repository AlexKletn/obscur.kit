Dock-to-edge layout, in the style of a WPF `DockPanel` (spec §4.1, §4.2).

Children are docked, in order, to the edges of a shrinking rectangle. Each
child claims a band along its assigned edge; that band is removed from the
remaining space, and the next child docks against the smaller rectangle that
is left. A [`Fill`](DockSide::Fill) child takes all remaining space.

# Configuration

This is a **self-configured** algorithm (the model has no dock-side field):
`sides` is parallel to the children by index. If a child has no corresponding
entry, it defaults to [`Fill`](DockSide::Fill); the last child conventionally
fills the centre when its side is `Fill`.

# Algorithm

A `remaining` rectangle starts at the padded content area. For each child in
order, looking up its side (defaulting to `Fill`):

* **Top** / **Bottom** — measured with its width fixed to the remaining width
  and its natural height (up to the remaining height); arranged flush against
  that edge; `remaining` shrinks vertically by the band height.
* **Left** / **Right** — measured with its height fixed to the remaining
  height and its natural width (up to the remaining width); arranged flush
  against that edge; `remaining` shrinks horizontally by the band width.
* **Fill** — measured under the whole `remaining` rectangle and arranged to
  cover it; `remaining` collapses to zero, so any later children get no
  space.

The node size is the bounding box of the docked children plus the container
`padding`, clamped into `constraints`.

# Out of scope

`LastChildFill` toggling, per-child dock spacing, and right-to-left edge
remapping are not modelled; docking is strictly in child order against the
four physical edges with a trailing fill.
