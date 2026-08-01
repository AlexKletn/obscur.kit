The same fill, but with the clip riding on a [`SceneLayer`] instead of being
bracketed by a `PushClip` — the shape the coordinator actually emits.

`opacity` is a parameter because the CPU backend takes two different paths: a
layer below `1.0` rasterizes its subtree into an offscreen buffer and
composites it back, while a trivial layer is a passthrough. A layer clip has to
be honored on both.
