Returns an immutable snapshot of the current scene.

This is the scene as of the most recent [`render_styled`](Self::render_styled);
a renderer (or a test) consumes the snapshot's display list without
touching the mutable scene.
