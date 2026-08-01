An interior node of the lossless syntax tree (spec §2.8).

A node is labelled with a node-valued [`SyntaxKind`] and owns an ordered
list of [`SyntaxElement`] children (nodes and tokens interleaved).

## Lossless invariant

[`SyntaxNode::text`] reconstructs the exact source substring the node
covers by concatenating, in order, the text of every descendant token. For
the tree's root this reproduces the whole source byte-for-byte.
