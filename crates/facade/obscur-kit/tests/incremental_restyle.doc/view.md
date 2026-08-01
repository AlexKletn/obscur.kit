A flat list of `count` sibling elements, the last one a `.btn`.

Flat and last on purpose: the reach of a hover is the hovered node's subtree
plus its *following* siblings' subtrees, so hovering the last sibling of a flat
list has a reach of exactly one node. Anything else the style phase resolves is
work the invalidation set did not ask for.
