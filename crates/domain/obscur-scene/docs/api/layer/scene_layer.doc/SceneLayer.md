A compositing layer descriptor: the parameters under which a group of
content is composited (spec §5.1).

A `SceneLayer` carries **no content**. The content it applies to is a display
list supplied by the command model (a later sub-step); this type is only the
compositing state pushed around that content — its transform, optional clip,
opacity, blend mode, Porter-Duff operator, and any additional effects. It is
backend-independent and references no GPU or renderer types.

A layer whose parameters are all neutral needs no offscreen pass and can be
flattened by the renderer; see [`SceneLayer::is_trivial`].
