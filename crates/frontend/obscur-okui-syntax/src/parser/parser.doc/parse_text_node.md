`text := (run of non-`<`, non-`{` tokens)` → a [`TextNode`](SyntaxKind::TextNode).

Absorbs raw text, whitespace and any stray non-markup tokens so nothing
is lost. Always consumes at least one token when called on a non-markup
token, guaranteeing forward progress.
