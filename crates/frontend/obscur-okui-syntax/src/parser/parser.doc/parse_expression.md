`expression := '{' expr_tokens '}'` → an [`Expression`](SyntaxKind::Expression)
node. Captures every token up to the matching `}`, tracking brace depth.

Precondition: `peek() == LBrace`.
