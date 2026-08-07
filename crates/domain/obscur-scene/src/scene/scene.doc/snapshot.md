Produces an immutable [`SceneSnapshot`] of the current state.

The snapshot shares the root display list (an [`Arc`](std::sync::Arc) bump, not
a copy), captures the current generation, and computes the statistics; it is
thereafter independent of this scene and unaffected by later mutation, because
`set_root` replaces the handle rather than mutating the list behind it.
