Returns the namespace prefix — the [`Ident`](SyntaxKind::Ident) immediately
before the [`Colon`](SyntaxKind::Colon) — for an event-style attribute
(`on` in `on:click`), or `None` when the attribute has no colon.
