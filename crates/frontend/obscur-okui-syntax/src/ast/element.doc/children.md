Iterates the element's content children — the direct child nodes between
its open and close tags.

Only content-shaped kinds are returned ([`Element`](SyntaxKind::Element),
[`Conditional`](SyntaxKind::Conditional), [`ForLoop`](SyntaxKind::ForLoop),
[`Expression`](SyntaxKind::Expression), [`TextNode`](SyntaxKind::TextNode));
the structural [`OpenTag`](SyntaxKind::OpenTag) and
[`CloseTag`](SyntaxKind::CloseTag) nodes are skipped. The nodes are cloned
so the returned iterator is owned and outlives this borrow.
