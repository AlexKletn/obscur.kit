`close := '<' '/' Ident '>'` → a [`CloseTag`](SyntaxKind::CloseTag) node.
Reports a diagnostic (but still recovers) when the close tag is absent or
its name does not match `expected`.
