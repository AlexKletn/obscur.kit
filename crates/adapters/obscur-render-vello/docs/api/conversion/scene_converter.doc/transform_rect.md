Maps an axis-aligned `rect` (a filter layer's local clip bounds) through the
accumulated transform `a` and returns the bounding box of the result, in `a`'s
target (device) space. For the usual scale + translate `a` (root device-scale,
scroll offset, ancestor transforms) the box is exact; for a rotation it is the
enclosing AABB, which still contains the whole filtered subtree. At
`a == identity` it returns `rect` unchanged, keeping the scale-1 path
byte-identical.
