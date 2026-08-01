Returns the [`ComputedStyle`] resolved for `node` on the last produced
frame, or [`None`] when the node was never styled (absent from the tree, or
a frame with a skipped style phase never resolved it).

This exposes the post-cascade computed value a host (or test) can inspect —
e.g. to observe that an author sheet's declaration overrode the UA default
sheet's for the same property (§4.14).
