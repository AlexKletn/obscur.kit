A subtree's opening `PushLayer` for a CSS `filter: blur(r)` group layer,
sorted before **every** other bracket at the same document position so the
blur is the OUTERMOST subtree wrapper (outside the transform, opacity, and
clip). The Vello renderer's owned gaussian pass then operates over the
fully-composited element. Below
[`TRANSFORM_PUSH_ORDER`](Self::TRANSFORM_PUSH_ORDER).
