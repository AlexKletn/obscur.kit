Reads a node's computed `background-image: url(...)` raw URL/path string, if
present.

Returns [`None`] when the node has no computed style, no `background-image`, or
a non-`url()` `background-image` (a gradient) — none of which paints an image.
The URL is the key the runtime resolves against its image cache.
