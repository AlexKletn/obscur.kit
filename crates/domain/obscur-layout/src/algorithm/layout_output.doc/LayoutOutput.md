The result a [`LayoutAlgorithm`] produces for a single node: the node's own
border-box size, plus the arranged box of each child.

[`LayoutAlgorithm`]: super::LayoutAlgorithm

This bundles the two outputs of the §4.2 measure/arrange contract for one
node: `size` is the node's resolved **border-box** size (the *measure*
result), and `child_boxes[i]` is child `i`'s [`LayoutBox`] (the *arrange*
result). Child boxes are expressed **in this node's coordinate space**: each
origin is relative to this node's origin and is already offset into the
node's content box (past its border and padding). The runtime translates them
into their parent's space as it walks back up the tree.

`child_boxes` is parallel to the `children` slice the algorithm was given:
`child_boxes[i]` positions `children[i]`.

# The third output: what a child's percentages resolve against

A child's box says where the child *is*; it does not say what containing
block the child's own `width: 50%` resolves against. For block flow and flex
those coincide with the parent's content box and the runtime supplies it
unaided, but a grid item's containing block is its grid area
(`specs/w3c/css-grid-2.txt:712`) — a fact only the grid algorithm holds.
[`child_percent_basis`](Self::child_percent_basis) is that fact, per child and
per axis; it defaults to [`ChildPercentBasis::UNSTATED`], so every algorithm
that does not set it behaves exactly as it did.
