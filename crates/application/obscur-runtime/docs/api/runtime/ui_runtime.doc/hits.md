The z-order-aware hit-test index rebuilt from `boxes` after each frame
that lays out, and the stacking-order resolution over it (§4.7).

One field rather than a bare index, because building it and ranking its
candidates are two halves of one stacking order — see [`HitTesting`], which
is why a box cannot be indexed under one depth and ranked under another.
