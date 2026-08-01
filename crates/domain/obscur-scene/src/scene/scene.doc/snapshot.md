Produces an immutable [`SceneSnapshot`] of the current state.

The snapshot clones the root display list, captures the current
generation, and computes the statistics; it is thereafter independent of
this scene and unaffected by later mutation.
