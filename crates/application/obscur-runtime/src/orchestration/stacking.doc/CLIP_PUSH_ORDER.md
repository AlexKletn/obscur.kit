A subtree's opening `PushClip`, sorted before the node's own paint at the
same document position. Pre-order enter indices are unique per node, so
pushes never collide; any value below [`PAINT_ORDER`](Self::PAINT_ORDER)
works.
