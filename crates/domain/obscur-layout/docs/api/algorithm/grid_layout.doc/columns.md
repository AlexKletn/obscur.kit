The number of columns (always at least `1`).

This reports the *explicit* column count (the explicit tracks plus one copy
of any auto-repeat pattern); the real per-content column count — which
depends on how many auto-repeat copies fit the container — is computed
during [`layout`](LayoutAlgorithm::layout).
