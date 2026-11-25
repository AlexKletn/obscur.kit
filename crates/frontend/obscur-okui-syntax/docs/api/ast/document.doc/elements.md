Iterates the document's top-level elements.

Every element-shaped child node — a plain [`Element`](SyntaxKind::Element)
as well as the element-shaped [`Conditional`](SyntaxKind::Conditional) and
[`ForLoop`](SyntaxKind::ForLoop) constructs — is cast to [`Element`];
non-element children (expressions, text) are skipped.
