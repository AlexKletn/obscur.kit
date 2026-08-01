[`run`] for a builder that reads the frame's [`FrameInfo`].

The runner owns the window, so it owns the only authoritative viewport: it keeps
the current one in a cell, refreshes it whenever the surface changes, and hands
the builder a snapshot each frame. That is what lets an app window its own list
to what is visible instead of building a node per item.
