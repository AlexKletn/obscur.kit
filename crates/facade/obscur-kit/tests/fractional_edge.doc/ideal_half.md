How wide a true circle of this radius gets anywhere inside the pixel row
`[y, y + 1]`, and how wide it is guaranteed to be everywhere inside it.

A rasterizer reaches a pixel when the shape touches any part of it, so the row's
*widest* point is what bounds how far paint may legitimately spread, and the
row's *narrowest* point is what it must cover at minimum. Sampling the circle at
one y instead would call the correct top and bottom rows over-wide by several
pixels.
