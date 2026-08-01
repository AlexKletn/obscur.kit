What a frame leaves behind for a caller to read: the retained tree and the commit
version that stamps it, the scene and its snapshot, the viewport-relative box
projection, and the identity every one of those readings is attributed to.

`layout_boxes` is here rather than with scrolling because it is a *reading*, not
a write: the projection it returns is the scroll offsets folded into the arranged
boxes, refreshed by the frame that settled them. `runtime_id` and `id` are the
same reading under two names, both public because two generations of caller
spell it differently.
