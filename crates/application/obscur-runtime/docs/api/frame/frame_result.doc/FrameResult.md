The outcome of one [`FramePipeline`](super::FramePipeline) run.

A `FrameResult` pairs the produced frame's identity with the immutable
[`SceneSnapshot`] a renderer consumes, plus a set of flags recording which
pipeline phases did observable work. The snapshot is the hand-off contract:
a renderer reads [`FrameResult::snapshot`] and honours its display list; it
never touches the mutable scene the pipeline built it from (spec §5.2).

The flags are diagnostic — they let the runtime and its instrumentation see
what a frame actually did (whether the tree changed, styles resolved, layout
ran, and a scene was composed) without re-deriving it.
