Which edge or corner of the window a resize region drives (spec §6.4).

`ResizeEdge` is a tightly-coupled part of [`ResizeRegion`](super::ResizeRegion): it names the grab
direction of a resize area and has no meaning apart from it, so it is
colocated here rather than given its own module.
