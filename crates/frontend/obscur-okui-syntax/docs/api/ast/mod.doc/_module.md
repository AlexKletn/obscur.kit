Typed AST: strongly-typed wrappers over the lossless `.okui` syntax tree
(spec §2.2–§2.12).

Each wrapper (rowan-style) owns a [`SyntaxNode`](crate::syntax::SyntaxNode) of
a specific kind and exposes typed accessors that read its child nodes and
tokens. Every wrapper provides `cast` (checked construction from a node) and
`syntax` (borrow of the underlying node); the lossless tree remains the
single source of truth, so these views add structure without copying content.
