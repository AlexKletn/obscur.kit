Drops the retained key of every node in `nodes`, so none of them can be
served this pass.

The boxes are deliberately **kept**: `nodes` is
[`LayoutEviction::arranges`](super::layout_eviction::LayoutEviction::arranges), which is upward-closed, so a node that
survives with its key has no evicted descendant, and its subtree's retained
boxes are exactly what a full pass would rewrite.
