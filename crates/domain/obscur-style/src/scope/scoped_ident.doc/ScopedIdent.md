A local identifier (for example a class name) qualified by the scope that
owns it (§4.15).

Because equality and hashing consider *both* fields, `.item` in one module
and `.item` in another are distinct [`ScopedIdent`]s — this is how module
isolation avoids requiring local identifiers to be globally unique.
`Copy + Eq + Hash` so it can key maps directly.
