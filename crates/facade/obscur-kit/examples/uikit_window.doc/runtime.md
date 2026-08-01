The embeddable runtime that owns the retained tree, scene, scheduler, and
live interaction state. Its reactive view builder reads the shared
`active_theme` to rebuild `uikit::view(i)`; its per-tab command handlers
mutate that same index.
