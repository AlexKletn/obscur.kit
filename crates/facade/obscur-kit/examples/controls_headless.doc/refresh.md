Rebuilds the view from the current state and runs one real frame through the
runtime (reconcile → match → layout → compose), refreshing the retained tree,
layout boxes, and spatial index so the next hit-test sees the current state.
