The table index of a symbol minted by `from_index`, or `None` for a reserved
symbol (top bit clear), which carries no index at all.

`named` mints into the same high namespace, so a named symbol also yields a
number here — a hash, not an index. A table must therefore treat the answer as
a *candidate* and bounds-check it, which is what makes a foreign symbol read
as "not mine" rather than as somebody else's row.
