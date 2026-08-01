Resolves every reference in `document` against `table`, returning the
accumulated errors (empty when the document is fully resolved).

The table is used as a mutable cursor: loop bodies push and pop child
scopes as they are entered and left, so on return the cursor is back at
the scope it started in.
