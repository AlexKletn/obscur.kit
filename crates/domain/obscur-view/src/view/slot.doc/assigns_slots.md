Whether any node in `nodes` carries a `slot="…"` assignment.

The reconciler's gate for retaining a component host's authored children:
the desired tree is gone by the time a component renders, so the content has
to be kept somewhere, and keeping *every* element's children would duplicate
the whole desired tree once per level of depth. A document that authors no
slot therefore pays nothing, and one that does pays for exactly the child
lists it assigned.
