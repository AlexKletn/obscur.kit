The styling. `100%` height rather than a fixed one, because the compositor —
not this sheet — decides the surface's real extent.

No `width` at all, deliberately: a `width: 100%` beside horizontal padding
makes the border box 28px wider than the surface, and the right padding ends
up off-screen with the last control pressed against the edge. The root fills
its surface anyway.
