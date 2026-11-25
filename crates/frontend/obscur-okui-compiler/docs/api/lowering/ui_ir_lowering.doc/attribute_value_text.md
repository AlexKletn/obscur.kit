Extracts an attribute's value as raw text: a quoted string's inner text, a
dynamic expression's source, or a bare `AttributeValue` token's text (as in
`on:click=submit`). Returns `None` when the attribute has no value.

`pub(crate)` because for an **event** attribute this string *is* the handler
reference — the name `name_binding::fallback_symbol` hashes into the command
symbol the runtime dispatches on. A checker that re-derived it would be
checking a name the IR might not carry, and a validator that admits one string
while the IR emits another is worse than no validator at all. One extraction,
so the checked name and the hashed name cannot drift.
