The scene font handle (`u32`) a resolved [`FontId`] paints under.

The registry hands ids out densely from `0` in registration order, and the
scene's font handle is that index — the identity every hand-wired host in this
repo re-derived for itself.
