A clip on a [`SceneLayer`] must confine drawing exactly as the same clip
bracketed by `PushClip` does, on both backends.

This is the divergence next door to §5-D, and a larger one: Vello honored
`SceneLayer::clip()` (`push_layer`'s bounding shape) and the CPU backend
dropped it on the floor, applying only the layer's opacity. It is not an edge
case — the coordinator brackets every `filter: blur()` / color-matrix group
with `with_clip`, so a real blurred element painted outside its clip on CPU and
inside it on Vello.

Both opacities are exercised because they are different code paths in the CPU
backend: a reduced-opacity layer rasterizes into an offscreen buffer, a trivial
one does not.
