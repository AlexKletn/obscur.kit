Lays `view` out in an existing `runtime` under `css` and returns the boxes.

The runtime is the caller's so it can carry state a single render cannot: a
registered component, and the mount count that says whether it ran.
