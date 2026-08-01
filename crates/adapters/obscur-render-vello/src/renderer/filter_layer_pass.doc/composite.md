Post-processes and composites every prepared layer onto `target_view`,
recording the work into the caller's `encoder` (so a caller may put its own
passes in the same submission).

Returns the colour-matrix scratch textures allocated along the way, so the
caller keeps them alive until that encoder is submitted.
