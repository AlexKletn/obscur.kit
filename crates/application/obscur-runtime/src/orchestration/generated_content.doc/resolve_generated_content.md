Resolves the generated decoration of `tree` under `sheets` (in `environment`)
into `out`, keyed by node id. Nodes with no decoration are absent.

`scope` says how much of `out` this pass owns. [`None`] rebuilds it from every
element in the tree. [`Some`] is the incremental restyle's dirty set: only
those nodes (and their ancestors) are resolved again, every other entry is the
previous frame's and stays exactly as it was, and entries for nodes the tree no
longer has are dropped — which is what the full pass would have done with them.
