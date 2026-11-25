Computes the reuse/create/remove decision for a parent's children.

`existing` is the parent's current ordered child ids (resolved against
`tree`); `desired` is the parent's desired child sequence. The returned
[`ChildMatching`] carries one decision per desired child in order, plus
the unmatched existing children to remove.
