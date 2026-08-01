A leaf of the lossless syntax tree (spec §2.8).

A `SyntaxToken` is the tree-level counterpart of a lexer [`Token`]: it keeps
the exact source slice and byte offset, but is labelled with a token-valued
[`SyntaxKind`] so it can live in a uniform tree of nodes and leaves.
