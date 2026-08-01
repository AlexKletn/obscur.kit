Emits `vector` into `paint`, placed within `region` per `object-fit` and drawn
in the node's computed `color` wherever the document says `currentColor`
(spec §5.17).

The document's outlines are left untouched: instead of transforming every
point, the placement is pushed as an [`AffineTransform`] that maps the
document's own space onto the placed tile, and popped afterwards. That keeps
the geometry exactly as parsed — vector through the pipeline — and costs one
command pair however many shapes the document has.

A document with a degenerate intrinsic size cannot be scaled onto a tile, so it
is skipped rather than divided by zero.
