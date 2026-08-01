The group blend that closes a CSS `opacity` / `mix-blend-mode` layer on the
software backend (spec §5.1, compositing-1 §5).

A non-trivial `PushLayer` rasterizes its whole subtree into an offscreen
[`tiny_skia::Pixmap`] pushed onto the renderer's target stack; this module owns
the other half of that bracket — folding the offscreen back onto the target
beneath it, exactly once, at the layer's opacity and blend mode. The step is a
unit because the rule it enforces is not "draw a pixmap": it is *the group is
composited as one*, and the root frame buffer is never consumed no matter how
malformed the display list that reached it.
