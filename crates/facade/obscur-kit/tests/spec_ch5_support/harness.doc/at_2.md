The straight-alpha `(r, g, b, a)` of the pixel at `(x, y)`.

The CPU backend's buffer is premultiplied, so this undoes the
premultiplication before reporting a channel — otherwise every assertion
about a colour under a non-opaque alpha would read the wrong number.
