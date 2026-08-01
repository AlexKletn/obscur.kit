Typed AST wrapper over an [`Expression`](SyntaxKind::Expression) syntax node
(spec §2.10).

An expression node covers a `{ … }` interpolation. Full expression parsing
is the compiler's job; this wrapper only exposes the raw inner source. Note
that the wrapped node keeps its surrounding braces — [`text`](Self::text)
strips the leading [`LBrace`](SyntaxKind::LBrace) and trailing
[`RBrace`](SyntaxKind::RBrace) tokens and concatenates the rest.
