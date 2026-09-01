Draw `source` over the whole of `target`, premultiplied.

One oversized triangle covers the target exactly, so the load op is `Load` and
nothing is cleared: every texel is written.
