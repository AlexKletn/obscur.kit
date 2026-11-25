Renders stale components, reconciles their output, and returns whether the
tree changed along with the resulting restyle roots.

An empty root list can accompany a removal when the styles of surviving
nodes remain valid. `None` leaves the style phase to take the full path.
Component output uses the same selector dependencies as the root reconcile.
