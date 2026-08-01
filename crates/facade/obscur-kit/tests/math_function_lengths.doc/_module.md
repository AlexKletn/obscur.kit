A math function is a `<length>`, everywhere one is allowed (CSS Values 4
§10.1) — through the whole stack, not only where it parses.

The defect these pin was two failures of the same rule, one at each end of
the pipeline, and each was invisible on its own:

1. **The frontend** rejected a math function as a component of a 1-4 value box
   shorthand. That does not degrade one side — it degrades the *whole*
   declaration to `Unsupported`, so `border-radius: min(3vmin, 14px)` left
   four square corners.
2. **The computed-value step** kept a `calc()` symbolic even after nothing in
   it was unknown. Every consumer that reads a length rather than an
   expression — the radius corners, `border-width`, the paint path — then read
   nothing and contributed zero.

Both are asserted here at the level a user would notice: painted geometry.
Neither would have failed a test of the parser alone, which is why these
render a frame rather than inspect a stylesheet.
