Squeezing a window and restoring it must not hang the application.

The bug this pins was the worst class the kit has had: shrinking a window to
its minimum and restoring it stopped the app dead — no drawing, and the
window could not even be closed. That last part is the tell. It was not a
slow frame or a wrong layout; the frame **never returned**, so the event loop
never got to handle anything again, including the close request.

The cause was a flex container whose padding exceeded the space it was given:
with no room left, the block axis is offered *unbounded*, and §9.7's
resolve-flexible-lengths loop grew an item toward infinity, clamped it
against an infinite maximum, and computed the violation as `inf − inf` — NaN.
NaN is false against every sign test, so the pass that decides what to freeze
froze nothing and the next pass recomputed the identical state, forever.

The whole sequence runs here, down to a zero viewport and back. A regression
does not fail this test — it hangs it, which the suite reports as a test that
never finishes. That is the honest signal for this class of defect.
