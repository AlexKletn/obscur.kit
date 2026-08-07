The root display list as the shared handle the scene actually holds.

A [`SceneSnapshot`] is taken at least once a frame, and the frame pipeline
takes one even for a frame that composed nothing. Holding the list behind an
[`Arc`](std::sync::Arc) makes that hand-off a refcount bump rather than a deep
copy of every command in the frame — the same trade `SharedPixels` makes for a
decoded image, and for the same reason: the list is immutable once composed, so
the copy bought nothing but its own cost.

Use [`Scene::root`] to read the list. This accessor exists for the callers that
need the *identity* of the list — a renderer holding one frame's list against
the next, or a test proving the snapshot did not copy.
