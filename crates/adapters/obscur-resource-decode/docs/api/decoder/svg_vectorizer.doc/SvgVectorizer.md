Converts SVG source into a [`SceneVector`] whose `currentColor` paints are
still unresolved (spec §5.17).

# The problem

`usvg` resolves `currentColor` **at parse time**: it reads the element's
`color` attribute, falling back to black, and bakes the result into the paint.
Nothing in its output distinguishes `fill="currentColor"` from `fill="black"`.
Following a theme change would therefore mean re-parsing the document with a
different injected colour — which is exactly what §5.17 forbids
("...allowing theme/state changes **without reparsing**").

# The approach

Parse **twice**, injecting a different `color` each time via `usvg`'s
stylesheet option, and compare the two results shape by shape:

* a paint that came out **different** was `currentColor` — it followed the
  injected colour, so it is recorded as [`VectorPaint::CurrentColor`] and
  resolved per frame from the computed `color`;
* a paint that came out **identical** was fixed by the document, and is
  recorded as [`VectorPaint::Solid`].

This is exact rather than heuristic: no sentinel colour can be mistaken for an
authored one, because detection turns on whether a paint *responded to* the
injection, not on what value it holds. The cost is one extra parse per
document, paid once at load — never per frame, which is the whole point.

It also gets a subtle case right for free: an SVG that declares its own
`color` (e.g. `<g color="blue"><path fill="currentColor"/></g>`) resolves
identically under both probes, so it is recorded as solid blue and the host's
computed `color` does **not** override it — matching how a browser treats a
nested `color` declaration.
