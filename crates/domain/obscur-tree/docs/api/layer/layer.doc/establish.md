Flags the layer's root node as a stacking-context root
([`NodeFlags::LAYER_ROOT`]) on `tree`.

Returns `true` if the root is live and was flagged; returns `false` (a
no-op) if the root id is stale or was never valid. It is panic-free and
mutates nothing when the root does not resolve.

This only records the establishing flag on the retained tree; actual
compositing lives in scene/render later.
