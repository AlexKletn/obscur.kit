A binding that captures one pointer to a node (spec §4.7 pointer capture).

While a capture is in force, every subsequent event for
[`pointer`](Self::pointer) routes to [`node`](Self::node) regardless of
hit-testing — letting a node that began an interaction (a drag, a slider
grab) keep receiving the pointer even after it leaves the node's bounds.
