Matches `sheets` against every element node in `tree` and returns the
resulting [`StyleInputs`].

`sheets` must be ordered by ascending cascade origin (system-default
first, inline last), the order [`StyleMatcher::match_element`] expects.
For each element node, the matched-and-cascaded
[`ResolvedDeclarations`](obscur_style::cascade::ResolvedDeclarations) is
recorded under the node's id only when it is non-empty, so nodes that
match no rule add no entry and resolve to their default/inherited style.
