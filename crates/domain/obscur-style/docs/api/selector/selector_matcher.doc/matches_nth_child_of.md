Decides `:nth-child(An+B of S)` (selectors-4 §13.3.1): the element must match
`S`, and its index is taken over the sibling list **filtered by `S` first**.

The filtered index is computed by walking preceding element siblings — the
only backward navigation `MatchElement` offers — and counting those that
match `S`; the element's own 1-based position in the filtered list is that
count plus one.
