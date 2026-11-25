The effective `filter` blur radius (logical px) bracketing a node's subtree, or
[`None`] when no positive-radius `blur()` op is authored (the byte-identical
no-blur path).

Takes the node's already-read op list rather than re-reading it: the blur, the
colour matrix and the drop-shadow list are three questions about ONE property,
and the paint walk reads it once per node and asks all three of the answer.

A `filter` may list several `blur()` ops; CSS applies them sequentially, but
combining separable gaussians is out of scope here, so the **maximum** authored
radius is used (multiple blurs on one element are uncommon; documented).
