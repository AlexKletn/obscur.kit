Captures an immutable snapshot of `scene`'s current state.

Shares the scene's root display list — a refcount bump, not a copy — and
captures its generation and statistics. This is what [`Scene::snapshot`] calls.
