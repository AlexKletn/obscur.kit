Typed AST wrapper over an [`Attribute`](SyntaxKind::Attribute) syntax node
(spec §2.3, §2.9).

An attribute has a name and, optionally, a value. Two value forms exist:

- a string literal — `attr="literal"` — surfaced by
  [`string_value`](Self::string_value) (quotes trimmed);
- a dynamic expression — `attr={expr}` — surfaced by
  [`value_expression`](Self::value_expression).

Event handlers use the `prefix:name` shape (for example `on:click`): the
attribute node then carries a [`Colon`](SyntaxKind::Colon) token, which
[`is_event`](Self::is_event) detects and [`namespace`](Self::namespace)
exposes.
