A SECOND view, structurally different from [`view`](Self::view), so that "did the
reconcile phase run" is observable.

Against an identical view a reconcile commits empty and `reconciled()` is false
either way — the assertion would pass without the plan doing anything at all. A
frame that reconciles adopts the extra child and the tree grows; a frame whose
plan skipped reconciliation leaves the tree exactly as it was.
