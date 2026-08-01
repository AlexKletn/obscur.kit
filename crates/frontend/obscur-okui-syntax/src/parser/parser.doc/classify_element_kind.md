Classifies an element by its tag name (see [`Parser`] docs). `If`/`ElseIf`/
`Else` → [`Conditional`](SyntaxKind::Conditional); `For`/`VirtualFor` →
[`ForLoop`](SyntaxKind::ForLoop); everything else → [`Element`](SyntaxKind::Element).
