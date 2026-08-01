Every [`SceneEffect`] `list` demands, deduplicated, in first-use paint order
(spec §5.2).

This is the one place the display list is read as a set of *effect demands*, so
a backend and a host agree on what a scene needs without either re-deriving it.
