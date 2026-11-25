Matches `element` the way [`match_element_in`](Self::match_element_in) does,
but consults `index` for the rules worth trying instead of trying all of them.

Byte for byte the same [`ResolvedDeclarations`] as the unindexed walk, for
every element and every sheet set. The index decides only which rules are
offered to [`SelectorMatcher::matches_list`](crate::selector::SelectorMatcher::matches_list);
the match itself, the specificity it yields, the `@media` condition and the
`source_order` each declaration is stamped with are unchanged, the last of
these because a candidate carries the global declaration index its block
begins at.

`index` must have been built from `sheets`. When it was not — or when the
element cannot enumerate its classes — this falls back to the unindexed walk
rather than answering from an index that does not fit.
