Splices each edit in order without generating new transactions — used by
[`undo`](Self::undo) and [`redo`](Self::redo). Each edit's range is in the
coordinate space produced by the edits before it, so they apply in order.
