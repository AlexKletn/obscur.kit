Builds the fill [`Brush`] for a node's computed `background-image` gradient
(linear / radial / conic), resolved over the box `bounds`, or [`None`] when the
node has no gradient background — the common (unauthored) case, which falls
back to the solid `background-color` path.

A stop-less gradient (never produced by the CSS lowering, but guarded here)
paints nothing. Exactly one of the three gradient kinds is present at a time
(`background-image` holds a single computed value).
