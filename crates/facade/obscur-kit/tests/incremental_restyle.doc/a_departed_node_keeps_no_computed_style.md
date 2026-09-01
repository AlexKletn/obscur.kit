Slides a 30-row window over 200 frames and asks whether the nodes the reconcile removed still answer `computed_style`.

`styling.doc/computed_style.md` says `None` for a node absent from the tree, so a non-zero count is both a leak and a contract the public API breaks. The incremental path retains its map across frames on purpose; what it must not retain is a key the tree no longer holds.
