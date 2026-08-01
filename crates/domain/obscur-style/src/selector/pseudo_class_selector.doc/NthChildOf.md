`:nth-child(An+B of S)` — position among the *inclusive siblings that
match `S`*, counting from the front (selectors-4 §13.3.1).

The element must itself match `S`; its index is then its position in the
`S`-filtered sibling list, not in the full one. The plain
[`NthChild`](Self::NthChild) form is kept separate rather than carrying an
`Option`, so the (common) no-`of` case allocates nothing.
