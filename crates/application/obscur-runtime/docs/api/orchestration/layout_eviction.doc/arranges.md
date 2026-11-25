The nodes whose `arrange` subtree can differ: the **upward closure** of
[`measures`](Self::measures), i.e. every `N` with
`subtree(N) ∩ measures ≠ ∅`.

`arrange(N, …)` writes an absolute box for every node in `N`'s subtree, so
its output can change exactly when some node in that subtree can — which is
what makes the closure, rather than `measures` itself, the right set. In a
well-formed tree the two are nearly equal (`measures` already carries each
dirty node's ancestor chain); the closure is taken explicitly anyway so this
rests on a *stated* property rather than on that coincidence surviving a
change to the clause above it or on [`MAX_DEPTH`] never truncating a chain.
