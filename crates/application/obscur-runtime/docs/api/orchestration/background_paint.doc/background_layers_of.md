Reads a node's computed multi-layer `background-image` list (two or more
comma-separated layers), in CSS author order (first = top), or [`None`] when
the node has a single flat `background-image` (a lone gradient/url) or none.

The single-layer case is deliberately *not* reported here — it stays on the
flat gradient/url paint path so a one-layer background is byte-identical.
