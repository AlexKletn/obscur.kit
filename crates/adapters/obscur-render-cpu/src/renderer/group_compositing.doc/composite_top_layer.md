Composites the top layer target of `targets` back onto the target beneath it
at `opacity` and `blend`, then drops it — the group blend for a CSS `opacity` /
`mix-blend-mode` layer (spec §5.1, compositing-1 §5).

The offscreen (which already holds the whole group's paint at full opacity) is
drawn onto the parent with a [`tiny_skia::PixmapPaint`] at `opacity` and the
`blend` operator, so the group blends **as one**: two overlapping opaque
children composite once, not twice (real group compositing, not per-element
alpha), and a non-`SourceOver` blend multiplies/screens/… the whole group onto
its backdrop. Does nothing when only the root target remains (a stray pop), so
the root is never consumed.
