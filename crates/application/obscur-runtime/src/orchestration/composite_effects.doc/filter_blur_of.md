The effective `filter` blur radius (logical px) bracketing a node's subtree, or
[`None`] when no positive-radius `blur()` op is authored (the byte-identical
no-blur path).

A `filter` may list several `blur()` ops; CSS applies them sequentially, but
combining separable gaussians is out of scope here, so the **maximum** authored
radius is used (multiple blurs on one element are uncommon; documented).
