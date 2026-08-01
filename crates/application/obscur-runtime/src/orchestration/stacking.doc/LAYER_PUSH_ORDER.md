A subtree's opening `PushLayer` (a CSS `opacity` group layer), sorted
*after* the `PushTransform` but *before* the `PushClip` at the same
document position, so the opacity layer nests inside the transform and
outside the rounded clip (transform → layer → clip). Between
[`TRANSFORM_PUSH_ORDER`](Self::TRANSFORM_PUSH_ORDER) and
[`CLIP_PUSH_ORDER`](Self::CLIP_PUSH_ORDER).
