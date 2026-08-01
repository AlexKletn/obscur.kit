Adjusts a content-space `point` by each scroll container it lies within,
walking `containers` **outermost-first** (they arrive depth-sorted): for a
container whose scrollport contains the running point, the container's
`scroll_top` is added to the point's `y` (undoing that container's
`translate(0, −scroll_top)`), and the walk continues with the adjusted point
so a nested container is tested in its own (outer-adjusted) coordinate space.

Returns the fully adjusted point (the layout-space coordinate the unscrolled
spatial index is keyed by) and the **innermost** container hit — exact for
the common single-container case; deeply nested scrollers rely on each
scrollport being tested after its ancestors' adjustment, a documented
best-effort.
