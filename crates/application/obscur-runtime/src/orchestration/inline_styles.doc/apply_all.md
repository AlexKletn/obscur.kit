Folds every node's `style` attribute into the freshly matched declarations.

Applying a block twice is harmless — same origin, same order, same value — which
is what lets the incremental restyle path re-apply to every styled node rather
than having to work out which ones it rematched.

It also sweeps the compile cache against the tree it just walked. An attribute
an application computes — a dragged panel writing `style="width: 412px"` every
frame — would otherwise leave one dead entry per value it passed through.
