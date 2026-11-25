What the scroll-folded projection is recomputed from: the retained tree (for
the ancestor walk), this frame's content-space boxes, and the scroll
containers that currently overflow.

The three arrive together because they must be *the same* three: folding one
frame's boxes along another frame's ancestor chain, or against containers
resolved from a third, produces a projection that describes no frame at all.
Grouping them also keeps the write operations to one extra parameter, so a
caller cannot pass the tree and forget the boxes.
