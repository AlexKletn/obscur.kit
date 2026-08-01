Whether any re-resolved node had **no** retained style, i.e. the walk added
an entry rather than replacing one.

Such a node has no "before" to compare against, so it is not in `previous`
and a consumer must not conclude anything from its absence there. Always
`false` when the caller's precondition holds (the retained map covers the
tree); it exists so that a caller which is wrong about that degrades to
conservative rather than to silently skipping work.
