Returns the canonical CSS keyword string for a reserved layout symbol.

This is the reverse of the `*_from_name` helpers, intended for diagnostics
and `DevTools`. It returns the canonical spelling (the flexbox spelling for
the `justify-content` / `align-items` start/end values). Returns [`None`] for
any symbol that is not one of the reserved layout keyword handles.
