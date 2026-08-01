Returns the inner expression source, without the surrounding braces.

The child tokens are concatenated in order, skipping a leading
[`LBrace`](SyntaxKind::LBrace) and a trailing [`RBrace`](SyntaxKind::RBrace)
if present. Braces nested inside the expression are preserved.
