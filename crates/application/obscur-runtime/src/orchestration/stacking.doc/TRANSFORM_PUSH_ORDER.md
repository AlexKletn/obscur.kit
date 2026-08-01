A subtree's opening `PushTransform`, sorted before both the `PushLayer`
and the `PushClip` at the same document position so a CSS 2D `transform`
brackets the whole subtree *outside* any opacity group layer or
rounded-corner clip (the transform is the outermost wrapper, save for a
blur layer). Between [`BLUR_PUSH_ORDER`](Self::BLUR_PUSH_ORDER) and
[`LAYER_PUSH_ORDER`](Self::LAYER_PUSH_ORDER).
