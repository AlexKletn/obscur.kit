Typed AST wrapper over a [`Conditional`](SyntaxKind::Conditional) syntax node
— an `<If>`, `<ElseIf>` or `<Else>` control-flow element (spec §2.11).

A conditional is element-shaped, so its tag name, attributes and content are
read through the shared [`Element`] view; this wrapper adds the
control-flow-specific accessors [`keyword`](Self::keyword),
[`condition`](Self::condition) and [`body`](Self::body). The branch condition
is carried by a `cond={…}` attribute (absent on a bare `<Else>`).
