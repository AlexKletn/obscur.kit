Extracts the root reference of an expression: its leading identifier.

Leading whitespace is skipped, then characters are taken while they are
alphanumeric or `_`, stopping at the first `.`, space or operator. Returns
`None` when the identifier is empty or starts with a digit (so it is not a
valid reference, e.g. a numeric literal like `42`).
