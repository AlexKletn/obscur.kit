The scrollable content height of `node` (logical px), measured from its
scrollport top down to the bottom-most descendant edge (CSS Overflow 3
§scrollable-overflow-region), never less than the scrollport height.

The walk descends the node's subtree but **stops at any nested scroll
container** (its own border box already bounds its overflow), so an inner
scroller's tall content does not inflate the outer scroller's extent.
