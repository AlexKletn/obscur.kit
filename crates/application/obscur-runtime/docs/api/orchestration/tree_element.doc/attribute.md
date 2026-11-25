Resolves only the attributes the retained tree preserves: `"id"` and the
authored `"type"`, both matched ASCII case-insensitively. Every other
attribute name returns `None` — see the type-level note on first-iteration
attribute limitations.

`[type=submit]` reads what the author wrote, not the element's tag: the tag is
the element's local name (a *type selector*'s subject, `local_name`), and the
two travel under separate reserved symbols precisely so
`<button type="submit">` can answer to `button` and to `[type=submit]` both.
